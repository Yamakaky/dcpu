use std::fmt;
use std::str::FromStr;
use std::error;

use num::FromPrimitive;

pub const MASK_OP: u16 = 0b11111;
pub const SHIFT_A: u16 = 10;
pub const SHIFT_B: u16 = 5;
pub const MASK_B: u16 = 0b11111;

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        DecodeError::description(self)
    }
}

quick_error!(
    #[derive(Debug)]
    pub enum DecodeError {
        BasicOp(val: u16) {
            description("invalid basic opcode")
            display("invalid basic opcode: {:#x}", val)
        }
        SpecialOp(val: u16) {
            description("invalid special opcode")
            display("invalid special opcode: {:#x}", val)
        }
    }
);

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Instruction<I> {
    /// op b a
    BasicOp(BasicOp, Value<I>, Value<I>),
    SpecialOp(SpecialOp, Value<I>)
}

impl Instruction<u16> {
    pub fn delay(&self) -> u16 {
        match *self {
            Instruction::BasicOp(op, b, a) => op.delay() + a.delay(true) + b.delay(false),
            Instruction::SpecialOp(op, a) => op.delay() + a.delay(true)
        }
    }

    pub fn encode(&self, output: &mut [u16]) -> u16 {
        match *self {
            Instruction::BasicOp(op, b, a) => {
                let mut i = 1;
                output[0] = op.encode();

                let (val, next) = a.encode(true);
                output[0] |= val << SHIFT_A;
                if let Some(n) = next {
                    output[i] = n;
                    i += 1;
                }

                let (val, next) = b.encode(false);
                output[0] |= val << SHIFT_B;
                if let Some(n) = next {
                    output[i] = n;
                    i += 1;
                }

                i as u16
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

    pub fn decode(data: &[u16; 3]) -> Result<(u16, Instruction<u16>),
                                             DecodeError> {
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

    pub fn is_if(&self) -> bool {
        match *self {
            Instruction::BasicOp(op, _, _) => op.is_if(),
            Instruction::SpecialOp(_, _) => false
        }
    }
}

impl fmt::Display for Instruction<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::BasicOp(op, b, a) => write!(f, "{:?} {:b}, {:o}", op, b, a),
            Instruction::SpecialOp(op, a) => write!(f, "{:?} {:o}", op, a)
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    A = 0x0,
    B = 0x1,
    C = 0x2,
    X = 0x3,
    Y = 0x4,
    Z = 0x5,
    I = 0x6,
    J = 0x7,
}
}

impl Register {
    pub fn offset(&self) -> u16 {
        *self as u16
    }
}

impl FromStr for Register {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Register, ParseError> {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Value<I> {
    Reg(Register),
    AtReg(Register),
    AtRegPlus(Register, I),
    Push,
    Peek,
    Pick(I),
    SP,
    PC,
    EX,
    AtAddr(I),
    Litteral(I)
}

impl Value<u16> {
    pub fn delay(&self, is_a: bool) -> u16 {
        match *self {
            Value::AtRegPlus(_, _) |
            Value::Pick(_) |
            Value::AtAddr(_) => 1,
            Value::Litteral(n) => if is_a && (n <= 0x1e || n == 0xffff) {
                // Let's say the compiler is smart ^^
                0
            } else {
                1
            },
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

    pub fn decode(val: u16, next: u16, is_a: bool) -> (u16, Value<u16>) {
        match val {
            x if x <= 0x17 => {
                let reg = Register::from_u16(x % 0x8).unwrap();
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
                 x <= 0x3f => (0, Value::Litteral(x.wrapping_sub(0x21))),
            _ => unreachable!()
        }
    }
}

impl fmt::Binary for Value<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Reg(r) => write!(f, "{:?}", r),
            Value::AtReg(r) => write!(f, "[{:?}]", r),
            Value::AtRegPlus(r, v) => write!(f, "[{:?} + {}]", r, v),
            Value::Pick(n) => write!(f, "PICK {}", n),
            Value::AtAddr(v) => write!(f, "[{}]", v),
            Value::Litteral(v) => write!(f, "{}", v),
            Value::Push => write!(f, "PUSH"),
            x => write!(f, "{:?}", x)
        }
    }
}

impl fmt::Octal for Value<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Reg(r) => write!(f, "{:?}", r),
            Value::AtReg(r) => write!(f, "[{:?}]", r),
            Value::AtRegPlus(r, v) => write!(f, "[{:?} + {}]", r, v),
            Value::Pick(n) => write!(f, "PICK {}", n),
            Value::AtAddr(v) => write!(f, "[{}]", v),
            Value::Litteral(v) => write!(f, "{}", v),
            Value::Push => write!(f, "POP"),
            x => write!(f, "{:?}", x)
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
        BasicOp::from_u16(op).ok_or(DecodeError::BasicOp(op))
    }

    pub fn is_if(&self) -> bool {
        match *self {
            BasicOp::IFB | BasicOp::IFC | BasicOp::IFE | BasicOp::IFN |
            BasicOp::IFG | BasicOp::IFA | BasicOp::IFL |
            BasicOp::IFU => true,
            _ => false
        }
    }
}

impl FromStr for BasicOp {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<BasicOp, ParseError> {
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

enum_from_primitive! {
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpecialOp {
    JSR = 0x01,
    INT = 0x08,
    IAG = 0x09,
    IAS = 0x0a,
    RFI = 0x0b,
    IAQ = 0x0c,
    HWN = 0x10,
    HWQ = 0x11,
    HWI = 0x12,
    LOG = 0x13,
    BRK = 0x14,
    HLT = 0x15,
}
}

impl SpecialOp {
    pub fn delay(&self) -> u16 {
        match *self {
            SpecialOp::BRK | SpecialOp::HLT => 0,
            SpecialOp::IAG | SpecialOp::IAS | SpecialOp::LOG => 1,
            SpecialOp::IAQ | SpecialOp::HWN => 2,
            SpecialOp::JSR | SpecialOp::RFI => 3,
            SpecialOp::INT | SpecialOp::HWQ | SpecialOp::HWI => 4
        }
    }

    pub fn encode(&self) -> u16 {
        *self as u16
    }

    pub fn decode(op: u16) -> Result<SpecialOp, DecodeError> {
        SpecialOp::from_u16(op).ok_or(DecodeError::SpecialOp(op))
    }
}

impl FromStr for SpecialOp {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<SpecialOp, ParseError> {
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
