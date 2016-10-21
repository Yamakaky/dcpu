use std::fmt;

use types::*;

impl fmt::Display for Instruction<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::BasicOp(op, b, a) => write!(f, "{:?} {:b}, {:o}", op, b, a),
            Instruction::SpecialOp(op, a) => write!(f, "{:?} {:o}", op, a)
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
