extern crate byteorder;
extern crate dcpu;
extern crate docopt;
extern crate nom;
extern crate rustc_serialize;
extern crate simplelog;

#[macro_use]
mod utils;

use std::io::{Read, Write};
use std::str;

use byteorder::WriteBytesExt;
use docopt::Docopt;
use nom::IResult::*;
use nom::HexDisplay;

use dcpu::assembler::{linker, parser};

const USAGE: &'static str = "
Usage:
  assembler [--no-cpp] [--ast] [--hex] [<file>] [-o <file>]
  assembler (--help | --version)

Options:
  --no-cpp      Disable gcc preprocessor pass.
  --ast         Show the file AST.
  --hex         Show in hexadecimal instead of binary.
  <file>        File to use instead of stdin.
  -o <file>     File to use instead of stdout.
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_no_cpp: bool,
    flag_ast: bool,
    flag_hex: bool,
    arg_file: Option<String>,
    flag_o: Option<String>,
}

fn line_number(raw_file: &[u8], raw_line: &[u8]) -> (usize, usize) {
    let offset = raw_file.offset(raw_line);
    assert!(offset < raw_file.len());
    let file = str::from_utf8(raw_file).unwrap();
    file.char_indices()
        .take_while(|&(i, _)| i <= offset)
        .map(|(_, c)| c)
        .fold((1, 1), |(line, row), c| {
            if c == '\n' {
                (line + 1, 0)
            } else {
                (line, row + 1)
            }
        })
}

fn main_ret() -> i32 {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info).unwrap();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let asm = {
        let mut asm = String::new();
        let mut input = utils::get_input(args.arg_file);
        input.read_to_string(&mut asm).unwrap();
        asm
    };

    let preprocessed = {
        if args.flag_no_cpp {
            asm
        } else {
            dcpu::assembler::preprocessor::preprocess(&asm).unwrap()
        }
    };
    let parsed = parser::parse(&preprocessed.as_bytes());
    let ast = match parsed {
        Done(ref i, ref o) if i.len() == 0 => o,
        Done(ref i, _) => {
            let (line, row) = line_number(&preprocessed.as_bytes(), i);
            die!(1,
                 "Unknown (line {}, row {}): \"{}\"",
                 line,
                 row,
                 str::from_utf8(i).unwrap().lines().next().unwrap());
        }
        e => die!(1, "Error: {:?}", e)
    };

    if args.flag_ast {
        die!(0, "{:?}", ast);
    }

    let bin = match linker::link(ast) {
        Ok(v) => v,
        Err(e) => die!(1, "Error: {:?}", e)
    };

    let mut output = utils::get_output(args.flag_o);

    if args.flag_hex {
        for n in bin {
            writeln!(output, "0x{:x}", n).unwrap();
        }
    } else {
        for n in bin {
            output.write_u16::<byteorder::LittleEndian>(n).unwrap();
        }
    }

    return 0;
}

fn main() {
    std::process::exit(main_ret());
}
