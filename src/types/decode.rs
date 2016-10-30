use enum_primitive::FromPrimitive;

use types::*;

error_chain! {
    errors {
        BasicOp(val: u16) {
            description("invalid basic opcode")
            display("invalid basic opcode: {:#x}", val)
        }
        SpecialOp(val: u16) {
            description("invalid special opcode")
            display("invalid special opcode: {:#x}", val)
        }
    }
}

impl Instruction<u16> {
    pub fn decode(data: &[u16; 3]) -> Result<(u16, Instruction<u16>)> {
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

impl Value<u16> {
    pub fn decode(val: u16, next: u16, is_a: bool) -> (u16, Value<u16>) {
        match val {
            x if x <= 0x17 => {
                let reg = Register::from_u16(x % 0x8).expect("Invalid reg id");
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

impl BasicOp {
    pub fn decode(op: u16) -> Result<BasicOp> {
        match BasicOp::from_u16(op) {
            Some(o) => Ok(o),
            None => try!(Err(ErrorKind::BasicOp(op))),
        }
    }
}

impl SpecialOp {
    pub fn decode(op: u16) -> Result<SpecialOp> {
        match SpecialOp::from_u16(op) {
            Some(o) => Ok(o),
            None => try!(Err(ErrorKind::SpecialOp(op))),
        }
    }
}
