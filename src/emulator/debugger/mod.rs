#[cfg(feature = "debugger-cli")]
mod completion;
#[cfg(feature = "debugger-cli")]
mod parser;

use std::collections::HashMap;
use std::iter::Iterator;
#[cfg(feature = "debugger-cli")]
use std::io;
use std::num::Wrapping;
#[cfg(feature = "debugger-cli")]
use std::path::Path;

use assembler;
use assembler::types::Expression;
use iterators;
use emulator::{cpu, device};
use emulator::device::Device;
#[cfg(feature = "debugger-cli")]
use emulator::debugger::parser::Command;
use types::Register;

error_chain! {
    links {
        cpu::Error, cpu::ErrorKind, Cpu;
    }
    errors {
        Breakpoint(i: usize, addr: u16, expr: Expression) {
            description("breakpoint triggered")
            display("breakpoint {} triggered at 0x{:0>4x} ({})",
                    i,
                    addr,
                    expr)
        }
    }
}

struct Breakpoint {
    addr: u16,
    expression: Expression,
}

pub struct Debugger {
    pub cpu: cpu::Cpu,
    devices: Vec<Box<device::Device>>,
    breakpoints: Vec<Breakpoint>,
    tick_number: u64,
    #[cfg(feature = "debugger-cli")]
    hooks: Vec<Command>,
    #[cfg(feature = "debugger-cli")]
    last_command: Option<Command>,
    log_litterals: bool,
    symbols: assembler::types::Globals,
}

impl Debugger {
    #[cfg(feature = "debugger-cli")]
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

    #[cfg(not(feature = "debugger-cli"))]
    pub fn new(mut cpu: cpu::Cpu, devices: Vec<Box<device::Device>>) -> Debugger {
        cpu.on_decode_error = cpu::OnDecodeError::Fail;
        Debugger {
            cpu: cpu,
            devices: devices,
            breakpoints: vec![],
            tick_number: 0,
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

    #[cfg(feature = "debugger-cli")]
    pub fn run<P: AsRef<Path>>(&mut self, history_path: P) {
        use rustyline::error::ReadlineError;
        use rustyline::Editor;

        let mut rl = Editor::new();
        rl.set_completer(Some(completion::DebuggerCompleter::new(&self.symbols)));
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
                        rl.add_history_entry(&line);
                        match parser::parse_command(&line) {
                            Ok(cmd) => {
                                Some(cmd.clone())
                            }
                            Err(e) => {
                                println!("{}", e);
                                continue
                            }
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
                Err(e) => println!("Error: {}", e),
            }
        }

        if let Err(e) = rl.save_history(&history_path) {
            println!("Error while saving the history file: {}", e);
        }
    }

    #[cfg(feature = "debugger-cli")]
    fn exec(&mut self, cmd: &Command) {
        match *cmd {
            Command::Step(n) => {
                for _ in 0..n {
                    if let Err(e) = self.step() {
                        println!("{}", e);
                    }
                }
            }
            Command::PrintRegisters => self.print_registers(),
            Command::Disassemble {ref from, size} =>
                self.disassemble(from, size),
            Command::Examine {ref from, size} => self.examine_expr(from, size),
            Command::Breakpoint(ref b) => {
                match b.solve(&self.symbols, &self.get_last_global()) {
                    Ok(addr) => self.breakpoints.push(Breakpoint {
                        addr: addr,
                        expression: b.clone(),
                    }),
                    Err(e) => println!("Invalid expression: {}", e),
                }
            }
            Command::ShowBreakpoints => self.show_breakpoints(),
            Command::DeleteBreakpoint(b) =>
                self.delete_breakpoint(b as usize),
            Command::Continue => if let Err(e) = self.continue_exec() {
                println!("{}", e);
            },
            Command::ShowDevices => self.show_devices(),
            Command::Hook(ref cmd) => if let Command::Hook(_) = **cmd {
                println!("You can't hook hooks!");
            } else {
                self.hooks.push(*cmd.clone());
            },
            Command::Logs => self.show_logs(),
            Command::M35fd(device_id, ref cmd) => {
                use emulator::device::m35fd::*;

                if let Some(m35fd) =
                    self.downcast_device::<M35fd>(device_id) {
                    match *cmd {
                        parser::M35fdCmd::Eject => {
                            if m35fd.eject().is_none() {
                                println!("This device is already empty");
                            }
                        }
                        parser::M35fdCmd::Load(ref path) => {
                            match Floppy::load(path) {
                                Ok(floppy) => m35fd.load(floppy),
                                Err(e) => println!("{}", e),
                            }
                        }
                    }
                }
            }
            Command::Stack(count) => self.examine(self.cpu.sp.0, count),
            Command::Symbols => self.show_symbols(),
            Command::List(n) => self.list(n),
        }
    }

    #[allow(dead_code)]
    pub fn step(&mut self) -> Result<()> {
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
            Err(e) => try!(Err(e)),
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn disassemble(&self, from: &Expression, size: u16) {
        let from = match from.solve(&self.symbols, &self.get_last_global()) {
            Ok(addr) => addr as usize,
            Err(e) => {
                println!("Invalid expression: {}", e);
                return;
            }
        };
        for i in iterators::U16ToInstruction::chain(self.cpu
                                                        .ram
                                                        .iter()
                                                        .cloned()
                                                        .skip(from))
                                             .take(size as usize) {
            println!("{}", i);
        }
    }

    #[allow(dead_code)]
    fn examine_expr(&self, from: &Expression, size: u16) {
        let from = match from.solve(&self.symbols, &self.get_last_global()) {
            Ok(addr) => addr,
            Err(e) => {
                println!("Invalid expression: {}", e);
                return;
            }
        };
        self.examine(from, size);
    }

    #[allow(dead_code)]
    fn examine(&self, from: u16, size: u16) {
        let to = from.checked_add(size).unwrap_or(0xffff);
        print!("0x{:0>4x}: ", from);
        for x in &self.cpu.ram[from..to] {
            print!("{:0>4x} ", x);
        }
        println!("");
    }

    #[allow(dead_code)]
    fn show_breakpoints(&self) {
        println!("Num    Address    Expression");
        for (i, b) in self.breakpoints.iter().enumerate() {
            println!("{:<4}   0x{:0>4x}     {}", i, b.addr, b.expression);
        }
    }

    pub fn delete_breakpoint(&mut self, b: usize) {
        if b < self.breakpoints.len() {
            self.breakpoints.remove(b);
        }
    }

    pub fn continue_exec(&mut self) -> Result<()> {
        loop {
            try!(self.step());

            if let Some((i, b)) = self.breakpoints
                                      .iter()
                                      .enumerate()
                                      .find(|&(_, x)| x.addr == self.cpu.pc.0) {
                try!(Err(ErrorKind::Breakpoint(i,
                                               b.addr,
                                               b.expression.clone())));
            }
        }
    }

    #[allow(dead_code)]
    fn show_devices(&self) {
        for (i, dev) in self.devices.iter().enumerate() {
            print!("Device {}: ", i);
            dev.inspect();
            println!("");
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn show_symbols(&self) {
        for symbol in self.symbols.keys() {
            println!("{}", symbol);
        }
    }

    #[allow(dead_code)]
    fn list(&self, n: u16) {
        let it = iterators::U16ToInstructionOffset::chain(
            self.cpu.ram.iter_wrap(self.cpu.pc.0).cloned()
        );
        let mut addr = self.cpu.pc;
        for (used, instr) in it.take(n as usize) {
            println!("0x{:0>4}: {}", addr, instr);
            addr += Wrapping(used);
        }
    }

    #[allow(dead_code)]
    fn downcast_device<D: Device>(&mut self,
                                  device_id: u16) -> Option<&mut D> {
        match self.devices.get_mut(device_id as usize) {
            Some(box_dev) => {
                let dev = box_dev.as_any().downcast_mut::<D>();
                if dev.is_none() {
                    println!("Device {} is not what you want", device_id);
                }
                dev
            }
            None => {
                println!("Invalid device id: {}", device_id);
                None
            }
        }
    }

    #[allow(dead_code)]
    fn get_last_global(&self) -> Option<String> {
        let mut i = 0;
        let mut last_global = None;
        for (name, s) in &self.symbols {
            if s.addr <= self.cpu.pc.0 && s.addr >= i {
                last_global = Some(name.clone());
                i = s.addr;
            }
        }
        last_global
    }
}
