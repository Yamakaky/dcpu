extern crate byteorder;
extern crate dcpu;
extern crate docopt;
extern crate rustc_serialize;

use std::io;
use std::io::Write;

use byteorder::ReadBytesExt;
use docopt::Docopt;

use dcpu::iterators::U16ToInstruction;

const USAGE: &'static str = "
Usage:
  disassembler [--ast]
  disassembler (--help | --version)

Options:
  --ast              Show the AST of the file.
  -h, --help         Show this message.
  --version          Show the version of disassembler.
";

#[derive(RustcDecodable)]
struct Args {
    flag_ast: bool,
}

struct IterU16<I> {
    pub input: I
}

impl<I: ReadBytesExt> Iterator for IterU16<I> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        self.input.read_u16::<byteorder::LittleEndian>().ok()
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let input = io::stdin();
    let mut output = io::stdout();

    for i in U16ToInstruction::chain(IterU16{input: input}) {
        if args.flag_ast {
            write!(output, "{:?}\n", i).unwrap();
        } else {
            write!(output, "{}\n", i).unwrap();
        }
    }
}
