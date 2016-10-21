mod nom_parser;

use std::str::{self, FromStr};
use std::result;

use nom::{HexDisplay, IResult};

use assembler::types::*;
pub use self::nom_parser::{expression, pos_number};

quick_error!(
    #[derive(Debug)]
    pub enum ParseError {
        BasicOp {
            description("invalid basic operator")
            display("invalid basic operator")
        }
        SpecialOp {
            description("invalid special operator")
            display("invalid special operator")
        }
        Register {
            description("invalid register")
            display("invalid register")
        }
    }
);

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

impl FromStr for BasicOp {
    type Err = ParseError;
    fn from_str(s: &str) -> result::Result<BasicOp, ParseError> {
        match s.to_uppercase().as_str() {
            "SET" => Ok(BasicOp::SET),
            "ADD" => Ok(BasicOp::ADD),
            "SUB" => Ok(BasicOp::SUB),
            "MUL" => Ok(BasicOp::MUL),
            "MLI" => Ok(BasicOp::MLI),
            "DIV" => Ok(BasicOp::DIV),
            "DVI" => Ok(BasicOp::DVI),
            "MOD" => Ok(BasicOp::MOD),
            "MDI" => Ok(BasicOp::MDI),
            "AND" => Ok(BasicOp::AND),
            "BOR" => Ok(BasicOp::BOR),
            "XOR" => Ok(BasicOp::XOR),
            "SHR" => Ok(BasicOp::SHR),
            "ASR" => Ok(BasicOp::ASR),
            "SHL" => Ok(BasicOp::SHL),
            "IFB" => Ok(BasicOp::IFB),
            "IFC" => Ok(BasicOp::IFC),
            "IFE" => Ok(BasicOp::IFE),
            "IFN" => Ok(BasicOp::IFN),
            "IFG" => Ok(BasicOp::IFG),
            "IFA" => Ok(BasicOp::IFA),
            "IFL" => Ok(BasicOp::IFL),
            "IFU" => Ok(BasicOp::IFU),
            "ADX" => Ok(BasicOp::ADX),
            "SBX" => Ok(BasicOp::SBX),
            "STI" => Ok(BasicOp::STI),
            "STD" => Ok(BasicOp::STD),
            _     => Err(ParseError::BasicOp)
        }
    }
}

impl FromStr for SpecialOp {
    type Err = ParseError;

    fn from_str(s: &str) -> result::Result<SpecialOp, ParseError> {
        match s.to_uppercase().as_str() {
            "JSR" => Ok(SpecialOp::JSR),
            "INT" => Ok(SpecialOp::INT),
            "IAG" => Ok(SpecialOp::IAG),
            "IAS" => Ok(SpecialOp::IAS),
            "RFI" => Ok(SpecialOp::RFI),
            "IAQ" => Ok(SpecialOp::IAQ),
            "HWN" => Ok(SpecialOp::HWN),
            "HWQ" => Ok(SpecialOp::HWQ),
            "HWI" => Ok(SpecialOp::HWI),
            "LOG" => Ok(SpecialOp::LOG),
            "BRK" => Ok(SpecialOp::BRK),
            "HLT" => Ok(SpecialOp::HLT),
            _ => Err(ParseError::SpecialOp)
        }
    }
}

impl FromStr for Register {
    type Err = ParseError;

    fn from_str(s: &str) -> result::Result<Register, ParseError> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Register::A),
            "B" => Ok(Register::B),
            "C" => Ok(Register::C),
            "I" => Ok(Register::I),
            "J" => Ok(Register::J),
            "X" => Ok(Register::X),
            "Y" => Ok(Register::Y),
            "Z" => Ok(Register::Z),
            _ => Err(ParseError::Register)
        }
    }
}
