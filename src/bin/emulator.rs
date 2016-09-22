extern crate byteorder;
extern crate dcpu;
extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate simplelog;

#[macro_use]
mod utils;

use docopt::Docopt;

use dcpu::emulator::Cpu;
use dcpu::emulator::Computer;
use dcpu::emulator::Debugger;
use dcpu::emulator::device::*;

const USAGE: &'static str = "
Usage:
  emulator [--debugger] [(-d <device>)...] [<file>]
  emulator (--help | --version)

Options:
  <file>             The binary file to execute.
  -d, --device       clock or keyscreen.
  --debugger         Launches the debugger.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_device: Option<Vec<String>>,
    arg_file: Option<String>,
    flag_debugger: bool,
}

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let rom = {
        let input = utils::get_input(args.arg_file);
        let mut rom = Vec::new();
        rom.extend(utils::IterU16{input: input});
        rom
    };

    let mut cpu = Cpu::default();
    cpu.load(&rom, 0);

    let devices = {
        let mut devices: Vec<Box<Device>> = vec![];
        if let Some(devs) = args.arg_device {
            for d in devs {
                match d.as_ref() {
                    "clock" => devices.push(Box::new(clock::Clock::new(100_000))),
                    "keyscreen" => {
                        let (screen_backend, kb_backend) = glium_backend::start();
                        devices.push(Box::new(keyboard::Keyboard::new(kb_backend)));
                        devices.push(Box::new(lem1802::LEM1802::new(screen_backend)));
                    }
                    _ => println!("Device \"{}\" unknown, ignoring", d),
                }
            }
        }
        devices
    };

    if args.flag_debugger {
        let mut debugger = Debugger::new(cpu, devices);
        debugger.run();
    } else {
        let mut computer = Computer::new(cpu, devices);

        loop {
            match computer.tick() {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
        }
    }
}
