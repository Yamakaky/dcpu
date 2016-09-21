mod parser;

use std::io;
use std::io::prelude::*;
use std::iter::Iterator;

use nom;

use cpu;
use device;
use iterators;
use debugger::parser::*;
use glium_backend;

pub struct Debugger {
    cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<u16>,
    tick_number: u64,
}

impl Debugger {
    pub fn new(mut cpu: cpu::Cpu, devices: Vec<Box<device::Device>>) -> Debugger {
        cpu.on_decode_error = cpu::OnDecodeError::Fail;
        Debugger {
            cpu: cpu,
            devices: devices,
            breakpoints: vec![],
            tick_number: 0,
        }
    }

    pub fn run(&mut self) {
        while let Some(cmd) = Self::get_command() {
            match cmd {
                Command::Step => {
                    let _ = self.step();
                }
                Command::PrintRegisters => self.print_registers(),
                Command::Disassemble {from, size} =>
                    self.disassemble(from, size),
                Command::Examine {from, size} => self.examine(from, size),
                Command::Breakpoint(b) => self.breakpoints.push(b),
                Command::ShowBreakpoints => self.show_breakpoints(),
                Command::DeleteBreakpoint(b) =>
                    self.delete_breakpoint(b as usize),
                Command::Continue => self.continue_exec(),
            }
        }
    }

    fn get_command() -> Option<Command> {
        let stdin = io::stdin();

        print!("> ");
        io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            match parser::parse_command(line.as_bytes()) {
                nom::IResult::Done(ref i, o) if i.len() == 0 => return Some(o),
                _ => println!("Unknown command: {}", line),
            }
            print!("> ");
            io::stdout().flush().unwrap();
        }
        println!("");
        None
    }

    fn step(&mut self) -> Result<(), ()> {
        use device::TickResult;
        self.tick_number += 1;
        for (i, device) in self.devices.iter_mut().enumerate() {
            match device.tick(&mut self.cpu, self.tick_number) {
                TickResult::Nothing => (),
                TickResult::Interrupt(int) => {
                    println!("Hardware interrupt from device {} with message {}",
                             i, int);
                    self.cpu.interrupts_queue.push_back(int);
                }
            }
        }
        match self.cpu.tick(&mut self.devices) {
            Ok(cpu::CpuState::Executing) => Ok(()),
            Ok(cpu::CpuState::Waiting) => self.step(),
            Err(e) => {
                println!("Cpu error: {}", e);
                Err(())
            }
        }
    }

    fn print_registers(&self) {
        let regs = &self.cpu.registers;
        println!(" A {:>4x} |  B {:>4x} |  C {:>4x}",
                 regs[0], regs[1], regs[2]);
        println!(" I {:>4x} |  J {:>4x}", regs[3], regs[4]);
        println!(" X {:>4x} |  Y {:>4x} |  Z {:>4x}",
                 regs[5], regs[6], regs[7]);

        println!("PC {:>4x} | SP {:>4x} | EX {:>4x} | IA {:>4x}",
                 self.cpu.pc, self.cpu.sp, self.cpu.ex, self.cpu.ia);
    }

    fn disassemble(&self, from: u16, size: u16) {
        for i in iterators::U16ToInstruction::chain(self.cpu
                                                        .ram
                                                        .iter()
                                                        .cloned()
                                                        .skip(from as usize))
                                             .take(size as usize) {
            println!("{}", i);
        }
    }

    fn examine(&self, from: u16, size: u16) {
        println!("{:?}", &self.cpu.ram[from as usize..(from + size) as usize]);
    }

    fn show_breakpoints(&self) {
        println!("Num    Address");
        for (i, b) in self.breakpoints.iter().enumerate() {
            println!("{:<4}   0x{:0>4x}", i, b);
        }
    }

    fn delete_breakpoint(&mut self, b: usize) {
        if b < self.breakpoints.len() {
            self.breakpoints.remove(b);
        }
    }

    fn continue_exec(&mut self) {
        loop {
            match self.step() {
                Ok(()) => (),
                Err(()) => return,
            }

            if let Some((i, addr)) = self.breakpoints
                                 .iter()
                                 .enumerate()
                                 .filter(|&(_, x)| *x == self.cpu.pc)
                                 .next() {
                println!("Breakpoint {} triggered at {}", i, addr);
                return;
            }
        }
    }
}
