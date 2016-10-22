use std::fmt;

use assembler::types::Globals;
use types::*;

impl fmt::Display for Instruction<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::BasicOp(op, b, a) => write!(f, "{:?} {:b}, {:o}", op, b, a),
            Instruction::SpecialOp(op, a) => write!(f, "{:?} {:o}", op, a)
        }
    }
}

impl Instruction<u16> {
    pub fn retrosolve(&self, globals: &Globals) -> String {
        match *self {
            Instruction::BasicOp(op, b, a) =>
                format!("{:?} {}, {}",
                        op,
                        b.retrosolve(globals, false),
                        a.retrosolve(globals, true)),
            Instruction::SpecialOp(op, a) =>
                format!("{:?} {}",
                        op,
                        a.retrosolve(globals, true)),
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

impl Value<u16> {
    pub fn retrosolve(&self, globals: &Globals, is_a: bool) -> String {
        let reverse = |addr| {
            for (sym, infos) in globals {
                if infos.addr == addr {
                    return format!("{} ({})", addr, sym.clone());
                }
            }
            format!("{}", addr)
        };
        match *self {
            Value::Reg(r) => format!("{:?}", r),
            Value::AtReg(r) => format!("[{:?}]", r),
            Value::AtRegPlus(r, v) => format!("[{:?} + {}]", r, v),
            Value::Pick(n) => format!("PICK {}", n),
            Value::AtAddr(v) => format!("[{}]", reverse(v)),
            Value::Litteral(v) => format!("{}", reverse(v)),
            Value::Push => if is_a {
                format!("POP")
            } else {
                format!("PUSH")
            },
            x => format!("{:?}", x)
        }
    }
}
