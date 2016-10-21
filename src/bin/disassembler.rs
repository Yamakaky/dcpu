extern crate dcpu;
#[cfg(feature = "bins")]
extern crate docopt;
#[macro_use]
extern crate log;
#[cfg(feature = "bins")]
extern crate rustc_serialize;
#[cfg(feature = "bins")]
extern crate simplelog;

#[macro_use]
mod utils;

use std::io::Write;

#[cfg(feature = "bins")]
use docopt::Docopt;

use dcpu::byteorder::{ReadBytesExt, LittleEndian};
use dcpu::iterators::U16ToInstruction;

#[cfg(feature = "bins")]
const USAGE: &'static str = "
Usage:
  disassembler [--ast] [<file>] [-o <file>]
  disassembler (--help | --version)

Options:
  --ast              Show the AST of the file.
  <file>             File to use instead of stdin.
  -o <file>          File to use instead of stdout.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[cfg(feature = "bins")]
#[derive(RustcDecodable)]
struct Args {
    flag_ast: bool,
    arg_file: Option<String>,
    flag_o: Option<String>,
}

#[cfg(feature = "bins")]
fn main_ret() -> i32 {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let mut input = match utils::get_input(args.arg_file) {
        Ok(input) => input,
        Err(e) => die!(1, "Error while opening the input: {}", e),
    };
    let mut output = match utils::get_output(args.flag_o) {
        Ok(o) => o,
        Err(e) => die!(1, "Error while opening the output: {}", e),
    };

    for i in U16ToInstruction::chain(input.iter_items::<u16, LittleEndian>()) {
        if args.flag_ast {
            writeln!(output, "{:?}", i).unwrap();
        } else {
            writeln!(output, "{}", i).unwrap();
        }
    }
    0
}

#[cfg(not(feature = "bins"))]
fn main_ret() -> i32 {
    "The feature \"bins\" must be activated to use this binary"
}

fn main() {
    std::process::exit(main_ret());
}
