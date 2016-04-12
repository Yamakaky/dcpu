extern crate dcpu;
extern crate docopt;
extern crate nom;
extern crate rustc_serialize;

use std::io::Read;

use docopt::Docopt;
use nom::IResult::*;

use dcpu::parser;

const USAGE: &'static str = "
Usage:
  assembler [--no-cpp] [--ast]
  assembler (--help | --version)

Options:
  --no-cpp      Disable gcc preprocessor pass.
  --ast         Show the file AST.
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(RustcDecodable)]
struct Args {
    flag_no_cpp: bool
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    let mut asm = String::new();
    std::io::stdin().read_to_string(&mut asm).unwrap();

    let preprocessed = {
        if args.flag_no_cpp {
            asm
        } else {
            dcpu::preprocessor::preprocess(&asm).unwrap()
        }
    };
    let parsed = parser::parse(&preprocessed.as_bytes());
    let ast = match parsed {
        Done(ref i, ref o) if i.len() == 0 => o,
        x => {
            println!("Error: {:?}", x);
            std::process::exit(1);
        }
    };
    println!("{:?}", ast);
}
