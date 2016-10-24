pub mod nom_parser;
pub mod combine_parser;

use std::str;

use nom::IResult;

use assembler::types::*;

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
            let (line, row) = nom_parser::line_number(source.as_bytes(), i);
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
