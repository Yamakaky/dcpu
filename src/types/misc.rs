use types::*;

impl Instruction<u16> {
    pub fn delay(&self) -> u16 {
        match *self {
            Instruction::BasicOp(op, b, a) => op.delay() + a.delay(true) + b.delay(false),
            Instruction::SpecialOp(op, a) => op.delay() + a.delay(true)
        }
    }

    pub fn is_if(&self) -> bool {
        match *self {
            Instruction::BasicOp(op, _, _) => op.is_if(),
            Instruction::SpecialOp(_, _) => false
        }
    }
}

impl Register {
    pub fn offset(&self) -> u16 {
        *self as u16
    }
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

    pub fn is_if(&self) -> bool {
        match *self {
            BasicOp::IFB | BasicOp::IFC | BasicOp::IFE | BasicOp::IFN |
            BasicOp::IFG | BasicOp::IFA | BasicOp::IFL |
            BasicOp::IFU => true,
            _ => false
        }
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
}
