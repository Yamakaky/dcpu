mod completion;
mod parser;

use std::collections::HashMap;
use std::iter::Iterator;
use std::io;
use std::path::Path;

use nom;

use assembler;
use iterators;
use emulator::{cpu, device};
use emulator::debugger::parser::*;
use types::Register;

pub struct Breakpoint {
    addr: u16,
    expression: Expression,
}

pub struct Debugger {
    cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<Breakpoint>,
    tick_number: u64,
    hooks: Vec<Command>,
    last_command: Option<Command>,
    log_litterals: bool,
    symbols: assembler::types::Globals,
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
            log_litterals: false,
            symbols: HashMap::new(),
        }
    }

    pub fn log_litterals(&mut self, enabled: bool) {
        self.log_litterals = enabled;
    }

    pub fn symbols(&mut self, symbols: assembler::types::Globals) {
        self.symbols = symbols;
    }

    pub fn run<P: AsRef<Path>>(&mut self, history_path: P) {
        use rustyline::error::ReadlineError;
        use rustyline::Editor;

        let mut rl = Editor::new();
        rl.set_completer(Some(completion::DebuggerCompleter));
        if let Err(e) = rl.load_history(&history_path) {
            if let ReadlineError::Io(io_err) = e {
                if io_err.kind() != io::ErrorKind::NotFound {
                    println!("Error while opening the history file: {}",
                             io_err);
                }
            }
        }

        loop {
            match rl.readline(">> ") {
                Ok(line) => {
                    let maybe_cmd = if line == "" {
                        self.last_command.clone()
                    } else {
                        match parser::parse_command(line.as_bytes()) {
                            nom::IResult::Done(i, ref o) if i.len() == 0 => {
                                rl.add_history_entry(&line);
                                Some(o.clone())
                            }
                            _ => None,
                        }
                    };

                    if let Some(cmd) = maybe_cmd {
                        self.exec(&cmd);
                        for cmd in self.hooks.clone() {
                            self.exec(&cmd);
                        }
                        self.last_command = Some(cmd);
                    } else {
                        println!("Unknown command: {}", line);
                    }
                }
                Err(ReadlineError::Interrupted) => (),
                Err(ReadlineError::Eof) => break,
                Err(err) => println!("Error: {:?}", err),
            }
        }

        if let Err(e) = rl.save_history(&history_path) {
            println!("Error while saving the history file: {}", e);
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
            Command::Breakpoint(ref b) => match b.solve(&self.symbols, &None) {
                Ok(addr) => self.breakpoints.push(Breakpoint {
                    addr: addr,
                    expression: b.clone(),
                }),
                Err(e) => println!("Invalid expression: {:?}", e),
            },
            Command::ShowBreakpoints => self.show_breakpoints(),
            Command::DeleteBreakpoint(b) =>
                self.delete_breakpoint(b as usize),
            Command::Continue => self.continue_exec(),
            Command::ShowDevices => self.show_devices(),
            Command::Hook(ref cmd) => if let Command::Hook(_) = **cmd {
                println!("You can't hook hooks!");
            } else {
                self.hooks.push(*cmd.clone());
            },
            Command::Logs => self.show_logs(),
        }
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
        println!("Tick number: {}", self.tick_number);
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
        println!("Num    Address    Expression");
        for (i, b) in self.breakpoints.iter().enumerate() {
            println!("{:<4}   0x{:0>4x}     {}", i, b.addr, b.expression);
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

            if let Some((i, b)) = self.breakpoints
                                      .iter()
                                      .enumerate()
                                      .find(|&(_, x)| x.addr == self.cpu.pc.0) {
                println!("Breakpoint {} triggered at 0x{:0>4x} ({})",
                         i,
                         b.addr,
                         b.expression);
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

    fn show_logs(&mut self) {
        for msg in &self.cpu.log_queue {
            if self.log_litterals {
                info!("LOG 0x{:0>4x}: {}", msg, self.cpu.get_str(*msg));
            } else {
                info!("LOG 0x{:0>4x}", msg);
            }
        }
        self.cpu.log_queue.clear();
    }
}
