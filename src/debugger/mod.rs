mod parser;

use std::io;
use std::io::prelude::*;
use std::iter::Iterator;

use nom;

use cpu;
use device;
use iterators;
use debugger::parser::*;

pub struct Debugger {
    cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<u16>,
}

impl Debugger {
    pub fn new(mut cpu: cpu::Cpu) -> Debugger {
        cpu.on_decode_error = cpu::OnDecodeError::Fail;
        Debugger {
            cpu: cpu,
            devices: vec![],
            breakpoints: vec![],
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
        match self.cpu.tick(&mut self.devices) {
            Ok(cpu::CpuState::Executing) => Ok(()),
            Ok(cpu::CpuState::Waiting) => self.step(),
            Err(e) => {
                println!("Cpu error: {}", e);
                Err(())
            }
        }
    }

    fn print_registers(&mut self) {
        let regs = &self.cpu.registers;
        println!(" A {:>4x} |  B {:>4x} |  C {:>4x}",
                 regs[0], regs[1], regs[2]);
        println!(" I {:>4x} |  J {:>4x}", regs[3], regs[4]);
        println!(" X {:>4x} |  Y {:>4x} |  Z {:>4x}",
                 regs[5], regs[6], regs[7]);

        println!("PC {:>4x} | SP {:>4x} | EX {:>4x} | IA {:>4x}",
                 self.cpu.pc, self.cpu.sp, self.cpu.ex, self.cpu.ia);
    }

    fn disassemble(&mut self, from: u16, size: u16) {
        for i in iterators::U16ToInstruction::chain(self.cpu
                                                        .ram
                                                        .iter()
                                                        .cloned()
                                                        .skip(from as usize))
                                             .take(size as usize) {
            println!("{}", i);
        }
    }

    fn examine(&mut self, from: u16, size: u16) {
        println!("{:?}", &self.cpu.ram[from as usize..(from + size) as usize]);
    }

    fn continue_exec(&mut self) {
        loop {
            match self.step() {
                Ok(()) => (),
                Err(()) => return,
            }

            if self.breakpoints.contains(&self.cpu.pc) {
                println!("Breakpoint triggered at {}", self.cpu.pc);
                return;
            }
        }
    }
}