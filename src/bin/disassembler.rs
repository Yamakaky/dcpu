extern crate byteorder;
extern crate dcpu;
extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate simplelog;

#[macro_use]
mod utils;

use std::io::Write;

use docopt::Docopt;

use dcpu::iterators::U16ToInstruction;

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

#[derive(RustcDecodable)]
struct Args {
    flag_ast: bool,
    arg_file: Option<String>,
    flag_o: Option<String>,
}

fn main_ret() -> i32 {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let input = match utils::get_input(args.arg_file) {
        Ok(input) => input,
        Err(e) => die!(1, "Error while opening the input: {}", e),
    };
    let mut output = match utils::get_output(args.flag_o) {
        Ok(o) => o,
        Err(e) => die!(1, "Error while opening the output: {}", e),
    };

    for i in U16ToInstruction::chain(dcpu::iterators::IterU16{input: input}) {
        if args.flag_ast {
            writeln!(output, "{:?}", i).unwrap();
        } else {
            writeln!(output, "{}", i).unwrap();
        }
    }
    0
}

fn main() {
    std::process::exit(main_ret());
}
