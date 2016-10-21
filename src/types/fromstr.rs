use std::str::FromStr;

use types::*;

error_chain! {
    errors {
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
}

impl FromStr for BasicOp {
    type Err = Error;

    fn from_str(s: &str) -> Result<BasicOp> {
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
            _     => try!(Err(ErrorKind::BasicOp)),
        }
    }
}

impl FromStr for SpecialOp {
    type Err = Error;

    fn from_str(s: &str) -> Result<SpecialOp> {
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
            _     => try!(Err(ErrorKind::SpecialOp)),
        }
    }
}

impl FromStr for Register {
    type Err = Error;

    fn from_str(s: &str) -> Result<Register> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Register::A),
            "B" => Ok(Register::B),
            "C" => Ok(Register::C),
            "I" => Ok(Register::I),
            "J" => Ok(Register::J),
            "X" => Ok(Register::X),
            "Y" => Ok(Register::Y),
            "Z" => Ok(Register::Z),
            _   => try!(Err(ErrorKind::Register))
        }
    }
}
