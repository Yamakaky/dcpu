extern crate dcpu;
#[cfg(feature = "bins")]
extern crate docopt;
#[macro_use]
extern crate error_chain;
#[cfg(feature = "bins")]
extern crate rustc_serialize;
#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "bins")]
extern crate simplelog;

#[macro_use]
mod utils;

use std::io::Read;
use std::str;

#[cfg(feature = "bins")]
use docopt::Docopt;

use dcpu::byteorder::{WriteBytesExt, LittleEndian};
use dcpu::assembler::{self, ResultExt};

#[cfg(feature = "bins")]
const USAGE: &'static str = "
Usage:
  assembler [options] [<file>]
  assembler (--help | --version)

Options:
  --no-cpp      Disable gcc preprocessor pass.
  --ast         Show the file AST.
  --hex         Show in hexadecimal instead of binary.
  --remove-unused  Remove unused labels and associated code.
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
    flag_remove_unused: bool,
    flag_symbols: Option<String>,
    arg_file: Option<String>,
    flag_o: Option<String>,
}

#[cfg(feature = "bins")]
quick_main!(|| -> assembler::Result<()> {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Info,
                                Default::default())
                          .chain_err(|| "log init failure")?;

    let version = option_env!("CARGO_PKG_VERSION").map(|s| s.into());
    let args: Args = Docopt::new(USAGE)
                            .map(|d| d.version(version))
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let asm = {
        let mut asm = String::new();
        let mut input = utils::get_input(args.arg_file).chain_err(||
            "input file opening"
        )?;
        input.read_to_string(&mut asm).chain_err(|| "input reading")?;
        asm
    };

    let preprocessed = {
        if args.flag_no_cpp {
            asm
        } else {
            assembler::preprocess(&asm)?
        }
    };
    let ast = assembler::parse(&preprocessed)?;

    if args.flag_ast {
        println!("{:?}", ast);
        return Ok(());
    }

    let ast = if args.flag_remove_unused {
        assembler::clean(ast)
    } else {
        assembler::print_unused(&ast);
        ast
    };

    let (bin, symbols) = assembler::link(&ast)?;

    let mut output = utils::get_output(args.flag_o).chain_err(||
        "Error while opening the output"
    )?;

    if args.flag_hex {
        for n in bin {
            writeln!(output, "0x{:x}", n).chain_err(|| "print error")?;
        }
    } else {
        output.write_all_items::<u16, LittleEndian>(&bin)
              .chain_err(|| "output error")?;
    }

    if let Some(path) = args.flag_symbols {
        write_symbols(path, &symbols)?;
    }

    Ok(())
});

#[cfg(not(feature = "bins"))]
quick_main!(|| -> assembler::Result<i32> {
    "The feature \"bins\" must be activated to use this binary"
});

#[cfg(feature = "serde_json")]
fn write_symbols(path: String,
                 symbols: &assembler::types::Globals) -> assembler::Result<()> {
    let mut o = utils::get_output(Some(path))
                      .chain_err(|| "Error while opening the symbol map file")?;
    serde_json::to_writer_pretty(&mut o, symbols).unwrap();
    Ok(())
}

#[cfg(not(feature = "serde_json"))]
fn write_symbols(_path: String, _symbols: &assembler::types::Globals) -> Result<()> {
    Err("Symbol map generation is disabled, activate the \"nightly\" feature.".into())
}
