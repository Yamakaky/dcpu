mod parser;

use std::io;
use std::io::prelude::*;
use std::iter::Iterator;

use nom;

use iterators;
use emulator::{cpu, device};
use emulator::debugger::parser::*;
use types::Register;

pub struct Debugger {
    cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<u16>,
    tick_number: u64,
    hooks: Vec<Command>,
    last_command: Option<Command>,
}

impl Debugger {
    pub fn new(mut cpu: cpu::Cpu, devices: Vec<Box<device::Device>>) -> Debugger {
        cpu.on_decode_error = cpu::OnDecodeError::Fail;
        Debugger {
            cpu: cpu,
            devices: devices,
            breakpoints: vec![],
            tick_number: 0,
            hooks: vec![],
            last_command: None,
        }
    }

    pub fn run(&mut self) {
        while let Some(cmd) = self.get_command() {
            self.exec(&cmd);
            for cmd in self.hooks.clone() {
                self.exec(&cmd);
            }
            self.last_command = Some(cmd);
        }
    }

    fn exec(&mut self, cmd: &Command) {
        match *cmd {
            Command::Step(n) => {
                for _ in 0..n {
                    let _ = self.step();
                }
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
            Command::ShowDevices => self.show_devices(),
            Command::Hook(ref cmd) => self.hooks.push(*cmd.clone()),
        }
    }

    fn get_command(&self) -> Option<Command> {
        let stdin = io::stdin();

        print!("> ");
        io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if line == "" {
                if let Some(ref cmd) = self.last_command {
                    return Some(cmd.clone());
                }
            }
            match parser::parse_command(line.as_bytes()) {
                nom::IResult::Done(ref i, ref o) if i.len() == 0 => return Some(o.clone()),
                _ => println!("Unknown command: {}", line),
            }
            print!("> ");
            io::stdout().flush().unwrap();
        }
        println!("");
        None
    }

    fn step(&mut self) -> Result<(), ()> {
        self.tick_number += 1;
        for (i, device) in self.devices.iter_mut().enumerate() {
            match device.tick(&mut self.cpu, self.tick_number) {
                device::TickResult::Nothing => (),
                device::TickResult::Interrupt(msg) => {
                    println!("Hardware interrupt from device {} with message {}",
                             i, msg);
                    self.cpu.hardware_interrupt(msg);
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
                 regs[Register::A], regs[Register::B], regs[Register::C]);
        println!(" I {:>4x} |  J {:>4x}",
                 regs[Register::I], regs[Register::J]);
        println!(" X {:>4x} |  Y {:>4x} |  Z {:>4x}",
                 regs[Register::X], regs[Register::Y], regs[Register::Z]);

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
        println!("{:?}", &self.cpu.ram[from..from + size]);
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

    fn show_devices(&self) {
        for (i, dev) in self.devices.iter().enumerate() {
            print!("Device {}: ", i);
            dev.inspect();
            println!("");
        }
    }
}
