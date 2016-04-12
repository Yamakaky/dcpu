extern crate dcpu;
extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate simplelog;

use docopt::Docopt;

use dcpu::cpu::Cpu;

const USAGE: &'static str = "
Usage:
  emulator [(-d <device>)...] <file>
  emulator (--help | --version)

Options:
  <file>             The binary file to execute.
  -d, --device       Des super devices.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: String,
    arg_device: Option<Vec<String>>
}

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Trace).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let cpu = Cpu::default();
}
