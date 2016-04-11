extern crate dcpu;
extern crate docopt;
extern crate rustc_serialize;

use std::io::Read;

use docopt::Docopt;

const USAGE: &'static str = "
Usage:
  assembler [--no-cpp]
  assembler (--help | --version)

Options:
  --no-cpp      Disable gcc preprocessor pass.
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

    let preprocessed = dcpu::preprocessor::preprocess(&asm).unwrap();
    println!("{}", preprocessed);
}
