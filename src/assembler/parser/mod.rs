mod nom_parser;

use std::str;

use nom::{HexDisplay, IResult};

use assembler::types::*;
pub use self::nom_parser::{expression, pos_number};

error_chain! {
    errors {
        Nom(e: String) {
            description("parsing error")
            display("parsing error: {}", e)
        }
    }
}

pub fn parse(source: &str) -> Result<Vec<ParsedItem>> {
    match nom_parser::parse(source.as_bytes()) {
        IResult::Done(i, o) => if i.len() == 0 {
            Ok(o)
        } else {
            let (line, row) = line_number(source.as_bytes(), i);
            try!(Err(format!("Unknown (line {}, row {}): \"{}\"",
                             line,
                             row,
                             str::from_utf8(i)
                                 .unwrap()
                                 .lines()
                                 .next()
                                 .unwrap())))
        },
        IResult::Error(e) => try!(Err(ErrorKind::Nom(format!("{}", e)))),
        e => try!(Err(format!("Error: {:?}", e))),
    }
}

pub fn line_number(raw_file: &[u8], raw_line: &[u8]) -> (usize, usize) {
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
