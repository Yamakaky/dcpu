use std::error::Error;
use std::fmt;
use std::str::FromStr;

use num::FromPrimitive;

pub const MASK_OP: u16 = 0b11111;
pub const SHIFT_A: u16 = 10;
pub const SHIFT_B: u16 = 5;
pub const MASK_B: u16 = 0b11111;

#[derive(Debug)]
pub enum DecodeError {
    BasicOp(u16),
    SpecialOp(u16)
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::BasicOp(ref e) => write!(f, "invalid basic opcode: {:x}", e),
            DecodeError::SpecialOp(ref e) => write!(f, "invalid special opcode: {:x}", e),
        }
    }
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        match *self {
            DecodeError::BasicOp(_) => "invalid basic opcode",
            DecodeError::SpecialOp(_) => "invalid special opcode"
        }
    }
}

#[derive(Debug)]
pub struct Ast {
    pub instructions: Vec<Instruction>
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.instructions.iter() {
            try!(write!(f, "{}\n", i));
        }
        write!(f, "")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// op b a
    BasicOp(BasicOp, Value, Value),
    SpecialOp(SpecialOp, Value)
}

impl Instruction {
    pub fn delay(&self) -> u16 {
        match *self {
            Instruction::BasicOp(op, b, a) => op.delay() + a.delay() + b.delay(),
            Instruction::SpecialOp(op, a) => op.delay() + a.delay()
        }
    }

    pub fn encode(&self, output: &mut [u16]) -> u16 {
        match *self {
            Instruction::BasicOp(op, b, a) => {
                let mut size = 1;
                output[0] = op.encode();

                let (val, next) = a.encode(true);
                output[0] |= (val << SHIFT_A);
                if let Some(n) = next {
                    output[1] = n;
                    size += 1;
                }

                let (val, next) = b.encode(false);
                output[0] |= (val << SHIFT_B);
                if let Some(n) = next {
                    output[2] = n;
                    size += 1;
                }

                size
            },
            Instruction::SpecialOp(op, v) => {
                let (a_bin, next) = v.encode(true);
                output[0] = op.encode() << SHIFT_B | (a_bin) << SHIFT_A;
                if let Some(n) = next {
                    output[1] = n;
                    2
                } else {
                    1
                }
            }
        }
    }

    pub fn decode(data: &[u16; 3]) -> Result<(u16, Instruction), DecodeError> {
        let op_bin = data[0] & MASK_OP;
        let a_bin = data[0] >> SHIFT_A;
        let b_bin = (data[0] >> SHIFT_B) & MASK_B;

        if op_bin == 0 {
            let op = try!(SpecialOp::decode(b_bin));
            let (used, a) = Value::decode(a_bin, data[1], true);
            Ok((1 + used, Instruction::SpecialOp(op, a)))
        } else {
            let op = try!(BasicOp::decode(op_bin));
            let (used_a, a) = Value::decode(a_bin, data[1], true);
            let (used_b, b) = Value::decode(b_bin, data[(1 + used_a) as usize], false);
            Ok((1 + used_a + used_b, Instruction::BasicOp(op, b, a)))
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::BasicOp(op, b, a) => write!(f, "{:?} {}, {}", op, b, a),
            Instruction::SpecialOp(op, a) => write!(f, "{:?} {}", op, a)
        }
    }
}

enum_from_primitive! {
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Register {
    A = 0x0,
    B = 0x1,
    C = 0x2,
    I = 0x3,
    J = 0x4,
    X = 0x5,
    Y = 0x6,
    Z = 0x7
}
}

impl Register {
    pub fn offset(&self) -> u16 {
        *self as u16
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Value {
    Reg(Register),
    AtReg(Register),
    AtRegPlus(Register, u16),
    Push,
    Peek,
    Pick(u16),
    SP,
    PC,
    EX,
    AtAddr(u16),
    Litteral(u16)
}

impl Value {
    pub fn delay(&self) -> u16 {
        // TODO : reg a + litteral
        match *self {
            Value::AtRegPlus(_, _) |
            Value::Pick(_) |
            Value::AtAddr(_) |
            Value::Litteral(_) => 1,
            _ => 0
        }
    }

    pub fn encode(&self, is_a: bool) -> (u16, Option<u16>) {
        match *self {
            Value::Reg(r) => (r.offset(), None),
            Value::AtReg(r) => (0x08 + r.offset(), None),
            Value::AtRegPlus(r, v) => (0x10 + r.offset(), Some(v)),
            Value::Push => (0x18, None),
            Value::Peek => (0x19, None),
            Value::Pick(v) => (0x1a, Some(v)),
            Value::SP => (0x1b, None),
            Value::PC => (0x1c, None),
            Value::EX => (0x1d, None),
            Value::AtAddr(v) => (0x1e, Some(v)),
            Value::Litteral(v) => {
                if is_a && (v <= 0x1e || v == 0xffff) {
                    (0x20 + v.wrapping_add(1), None)
                } else {
                    (0x1f, Some(v))
                }
            }
        }
    }

    pub fn decode(val: u16, next: u16, is_a: bool) -> (u16, Value) {
        match val {
            x if x <= 0x17 => {
                let reg = Register::from_i32((x % 0x8) as i32).unwrap();
                if x <= 0x07 {
                    (0, Value::Reg(reg))
                } else if x <= 0x0f {
                    (0, Value::AtReg(reg))
                } else {
                    (1, Value::AtRegPlus(reg, next))
                }
            },
            0x18 => (0, Value::Push),
            0x19 => (0, Value::Peek),
            0x1a => (1, Value::Pick(next)),
            0x1b => (0, Value::SP),
            0x1c => (0, Value::PC),
            0x1d => (0, Value::EX),
            0x1e => (1, Value::AtAddr(next)),
            0x1f => (1, Value::Litteral(next)),
            x if is_a &&
                 x >= 0x20 &&
                 x <= 0x3f => (0, Value::Litteral(x - 0x21)),
            _ => unreachable!()
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Reg(r) => write!(f, "{:?}", r),
            Value::AtReg(r) => write!(f, "[{:?}]", r),
            Value::AtRegPlus(r, v) => write!(f, "[{:?} + v]", r),
            Value::Pick(n) => write!(f, "PICK {}", n),
            Value::AtAddr(v) => write!(f, "[{}]", v),
            Value::Litteral(v) => write!(f, "{}", v),
            x => write!(f, "{:?}", x)
        }
    }
}

enum_from_primitive! {
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum BasicOp {
    SET = 0x01,
    ADD = 0x02,
    SUB = 0x03,
    MUL = 0x04,
    MLI = 0x05,
    DIV = 0x06,
    DVI = 0x07,
    MOD = 0x08,
    MDI = 0x09,
    AND = 0x0a,
    BOR = 0x0b,
    XOR = 0x0c,
    SHR = 0x0d,
    ASR = 0x0e,
    SHL = 0x0f,
    IFB = 0x10,
    IFC = 0x11,
    IFE = 0x12,
    IFN = 0x13,
    IFG = 0x14,
    IFA = 0x15,
    IFL = 0x16,
    IFU = 0x17,
    ADX = 0x1a,
    SBX = 0x1b,
    STI = 0x1e,
    STD = 0x1f
}
}

impl BasicOp {
    pub fn delay(&self) -> u16 {
        match *self {
            BasicOp::SET | BasicOp::AND | BasicOp::BOR | BasicOp::XOR |
            BasicOp::SHL | BasicOp::SHR | BasicOp::ASR => 1,
            BasicOp::DVI | BasicOp::DIV | BasicOp::MOD | BasicOp::MDI |
            BasicOp::ADX | BasicOp::SBX => 3,
            _ => 2
        }
    }

    pub fn encode(&self) -> u16 {
        *self as u16
    }

    pub fn decode(op: u16) -> Result<BasicOp, DecodeError> {
        BasicOp::from_i32(op as i32).ok_or(DecodeError::BasicOp(op))
    }
}

pub struct ParseBasicOpError;

impl FromStr for BasicOp {
    type Err = ParseBasicOpError;
    fn from_str(s: &str) -> Result<BasicOp, ParseBasicOpError> {
        match s {
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
            _     => Err(ParseBasicOpError)
        }
    }
}

enum_from_primitive! {
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum SpecialOp {
    JSR = 0x01,
    INT = 0x08,
    IAG = 0x09,
    IAS = 0x0a,
    RFI = 0x0b,
    IAQ = 0x0c,
    HWN = 0x10,
    HWQ = 0x11,
    HWI = 0x12
}
}

impl SpecialOp {
    pub fn delay(&self) -> u16 {
        match *self {
            SpecialOp::IAG | SpecialOp::IAS => 1,
            SpecialOp::IAQ | SpecialOp::HWN => 2,
            SpecialOp::JSR | SpecialOp::RFI => 3,
            SpecialOp::INT | SpecialOp::HWQ | SpecialOp::HWI => 4
        }
    }

    pub fn encode(&self) -> u16 {
        *self as u16
    }

    pub fn decode(op: u16) -> Result<SpecialOp, DecodeError> {
        SpecialOp::from_i32(op as i32).ok_or(DecodeError::SpecialOp(op))
    }
}
