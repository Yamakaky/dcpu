extern crate byteorder;
extern crate dcpu;
extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate simplelog;

#[macro_use]
mod utils;

use std::{time, thread};
use std::fs::File;
use std::io::{self, BufReader};
use std::io::prelude::*;

use docopt::Docopt;

use dcpu::emulator::Cpu;
use dcpu::emulator::Computer;
use dcpu::emulator::Debugger;
use dcpu::emulator::device::*;

const USAGE: &'static str = "
Usage:
  emulator [options] [(-d <device>)...] [<file>]
  emulator (--help | --version)

Options:
  <file>             The binary file to execute.
  --tps              Print the number of ticks by second
  --limit            Try to limit the tick rate to 100_000/s
  -d, --device       clock or keyscreen.
  --debugger         Launches the debugger.
  --log-map <file>   Mapping between LOG n and string
  --debug-history <file>   Use this file for the debugger history
                     [default: debug_history]
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_device: Option<Vec<String>>,
    arg_file: Option<String>,
    flag_log_map: Option<String>,
    flag_debugger: bool,
    flag_tps: bool,
    flag_limit: bool,
    flag_debug_history: String,
}

fn main_ret() -> i32 {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let rom = {
        let input = match utils::get_input(args.arg_file) {
            Ok(input) => input,
            Err(e) => die!(1, "Error while opening the input: {}", e),
        };
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
                    _ => die!(1, "Device \"{}\" unknown", d),
                }
            }
        }
        devices
    };

    if args.flag_debugger {
        let mut debugger = Debugger::new(cpu, devices);
        if let Some(path) = args.flag_log_map {
            let log_map = match load_log_map(&path) {
                Ok(map) => map,
                Err(e) => die!(1, "Some troube loading the log map: {}", e),
            };
            debugger.log_map(log_map);
        }
        debugger.run(args.flag_debug_history);
    } else {
        let mut computer = Computer::new(cpu, devices);
        let mut timer_tps = time::SystemTime::now();
        let mut timer_limit = time::SystemTime::now();
        let normal_tickrate = 100_000;
        let limit_check = 10_000;
        let tps_check = if args.flag_limit {
            normal_tickrate
        } else {
            10 * normal_tickrate
        };

        loop {
            match computer.tick() {
                Ok(_) => (),
                Err(e) => die!(1, "{}", e),
            }

            if args.flag_tps && computer.current_tick % tps_check == 0 {
                if let Ok(delay) = timer_tps.elapsed() {
                    let tps = tps_check * 1_000_000_000 / delay.subsec_nanos() as u64;
                    println!("{} tics per second, {}x speedup",
                             tps,
                             tps as f32 / normal_tickrate as f32);
                }

                timer_tps = time::SystemTime::now();
            }
            if args.flag_limit && computer.current_tick % limit_check == 0 {
                if let Ok(delay) = timer_limit.elapsed() {
                    let elapsed_ms = (delay.subsec_nanos() / 1_000_000) as u64;
                    let normal_duration = limit_check * 1_000 / normal_tickrate;
                    if elapsed_ms < normal_duration {
                        thread::sleep(time::Duration::from_millis(normal_duration
                                                                  - elapsed_ms));
                    }
                }

                timer_limit = time::SystemTime::now();
            }
        }
    }
    0
}

fn main() {
    std::process::exit(main_ret());
}

fn load_log_map(path: &str) -> io::Result<[Option<String>; 64]> {
    let file = BufReader::new(try!(File::open(path)));
    let mut log_map = [
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
    ];
    for line in file.lines() {
        let line = try!(line);
        let mut fields = line.splitn(2, ' ');
        if let Some(i) = fields.next() {
            if let Ok(i) = i.parse() {
                let i: usize = i;
                if let Some(s) = fields.next() {
                    log_map[i] = Some(s.into());
                }
            }
        }
    }
    Ok(log_map)
}
