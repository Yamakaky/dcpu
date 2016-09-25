mod parser;

use std::collections::BTreeSet;
use std::iter::Iterator;
use std::io;
use std::path::Path;

use nom;
use rustyline;

use iterators;
use emulator::{cpu, device};
use emulator::debugger::parser::*;
use types::Register;

struct DebuggerCompleter;

impl rustyline::completion::Completer for DebuggerCompleter {
    fn complete(&self, line: &str, pos: usize)
        -> rustyline::Result<(usize, Vec<String>)> {

        let break_chars = {
            let mut set = BTreeSet::new();
            set.insert(' ');
            set
        };
        let cmds = [
            "r", "x", "b", "s", "c",
            "devices",
            "disassemble",
            "breakpoints",
            "delete",
            "hook",
            "logs",
        ];
        let (i, word) = rustyline::completion::extract_word(line,
                                                            pos,
                                                            &break_chars);
        let completions = cmds.iter()
                              .filter(|cmd| cmd.starts_with(word))
                              .cloned()
                              .map(|s| (*s).into())
                              .collect();
        Ok((i, completions))
    }
}

pub struct Debugger {
    cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<u16>,
    tick_number: u64,
    hooks: Vec<Command>,
    last_command: Option<Command>,
    log_map: [Option<String>; 64]
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
            // Wow, such many None
            log_map: [None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None],
        }
    }

    pub fn log_map(&mut self, log_map: [Option<String>; 64]) {
        self.log_map = log_map;
    }

    pub fn run<P: AsRef<Path>>(&mut self, history_path: P) {
        use rustyline::error::ReadlineError;
        use rustyline::Editor;

        let mut rl = Editor::new();
        rl.set_completer(Some(DebuggerCompleter));
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
                            nom::IResult::Done(ref i, ref o) if i.len() == 0 => {
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
            Command::Breakpoint(b) => self.breakpoints.push(b),
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
                                 .filter(|&(_, x)| *x == self.cpu.pc.0)
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

    fn show_logs(&mut self) {
        for log in self.cpu.log_queue.drain(..) {
            print!("{}", log);
            if let Some(ref s) = self.log_map[log as usize] {
                print!(" - {}", s);
            }
            println!("");
        }
    }
}
