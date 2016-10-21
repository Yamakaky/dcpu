extern crate dcpu;
#[cfg(feature = "bins")]
extern crate docopt;
#[cfg(feature = "bins")]
extern crate rustc_serialize;
#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "bins")]
extern crate simplelog;

#[macro_use]
mod utils;

use std::io::{Read, Write};
use std::str;

#[cfg(feature = "bins")]
use docopt::Docopt;

use dcpu::byteorder::{WriteBytesExt, LittleEndian};
use dcpu::assembler;

#[cfg(feature = "bins")]
const USAGE: &'static str = "
Usage:
  assembler [options] [<file>]
  assembler (--help | --version)

Options:
  --no-cpp      Disable gcc preprocessor pass.
  --ast         Show the file AST.
  --hex         Show in hexadecimal instead of binary.
  --symbols <f>  Write the resolved symbols to this file.
  <file>        File to use instead of stdin.
  -o <file>     File to use instead of stdout.
  -h --help     Show this screen.
  --version     Show version.
";

#[cfg(feature = "bins")]
#[derive(Debug, RustcDecodable)]
struct Args {
    flag_no_cpp: bool,
    flag_ast: bool,
    flag_hex: bool,
    flag_symbols: Option<String>,
    arg_file: Option<String>,
    flag_o: Option<String>,
}

#[cfg(feature = "bins")]
fn main_ret() -> i32 {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let asm = {
        let mut asm = String::new();
        let mut input = match utils::get_input(args.arg_file) {
            Ok(input) => input,
            Err(e) => die!(1, "Error while opening the input: {}", e),
        };
        input.read_to_string(&mut asm).unwrap();
        asm
    };

    let preprocessed = {
        if args.flag_no_cpp {
            asm
        } else {
            assembler::preprocess(&asm).unwrap()
        }
    };
    let ast = match assembler::parse(&preprocessed) {
        Ok(o) => o,
        Err(e) => die!(1, "Error: {}", e),
    };

    if args.flag_ast {
        die!(0, "{:?}", ast);
    }

    let (bin, symbols) = match assembler::link(&ast) {
        Ok(v) => v,
        Err(e) => die!(1, "Error: {:?}", e)
    };

    let mut output = match utils::get_output(args.flag_o) {
        Ok(o) => o,
        Err(e) => die!(1, "Error while opening the output: {}", e),
    };

    if args.flag_hex {
        for n in bin {
            writeln!(output, "0x{:x}", n).unwrap();
        }
    } else {
        for n in bin {
            output.write_item::<u16, LittleEndian>(n).unwrap();
        }
    }

    if let Some(path) = args.flag_symbols {
        write_symbols(path, &symbols)
    } else {
        0
    }
}

#[cfg(not(feature = "bins"))]
fn main_ret() -> i32 {
    "The feature \"bins\" must be activated to use this binary"
}

fn main() {
    std::process::exit(main_ret());
}

#[cfg(feature = "serde_json")]
fn write_symbols(path: String, symbols: &assembler::types::Globals) -> i32 {
    match utils::get_output(Some(path)) {
        Ok(mut o) => serde_json::to_writer_pretty(&mut o, symbols).unwrap(),
        Err(e) => die!(1, "Error while opening the symbol map file: {}", e),
    }
    0
}

#[cfg(not(feature = "serde_json"))]
fn write_symbols(_path: String, _symbols: &assembler::types::Globals) -> i32 {
    die!(1, "Symbol map generation is disabled, activate the \"nightly\" feature.");
}
