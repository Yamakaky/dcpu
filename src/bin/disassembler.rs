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
  disassembler [--ast] [-i <file>] [-o <file>]
  disassembler (--help | --version)

Options:
  --ast              Show the AST of the file.
  -i <file>          File to use instead of stdin.
  -o <file>          File to use instead of stdout.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(RustcDecodable)]
struct Args {
    flag_ast: bool,
    flag_i: Option<String>,
    flag_o: Option<String>,
}

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let input = utils::get_input(args.flag_i);
    let mut output = utils::get_output(args.flag_o);

    for i in U16ToInstruction::chain(utils::IterU16{input: input}) {
        if args.flag_ast {
            writeln!(output, "{:?}", i).unwrap();
        } else {
            writeln!(output, "{}", i).unwrap();
        }
    }
}
