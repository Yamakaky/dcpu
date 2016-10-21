use types::*;

impl Instruction<u16> {
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
}

impl Value<u16> {
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
}

impl BasicOp {
    pub fn encode(&self) -> u16 {
        *self as u16
    }
}

impl SpecialOp {
    pub fn encode(&self) -> u16 {
        *self as u16
    }
}
