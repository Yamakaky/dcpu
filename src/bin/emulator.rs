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

use dcpu::cpu::Cpu;

const USAGE: &'static str = "
Usage:
  emulator [(-d <device>)...] [-i <file>]
  emulator (--help | --version)

Options:
  <file>             The binary file to execute.
  -d, --device       Des super devices.
  -i <file>          File to use instead of stdin.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_device: Option<Vec<String>>,
    flag_i: Option<String>,
}

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let rom = {
        let input = utils::get_input(args.flag_i);
        let mut rom = Vec::new();
        rom.extend(utils::IterU16{input: input});
        rom
    };

    let mut cpu = Cpu::default();
    cpu.load(&rom, 0);

    let mut devices = Vec::new();

    loop {
        match cpu.tick(&mut devices) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}
