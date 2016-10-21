pub mod decode;
mod encode;
mod fmt;
mod fromstr;
mod misc;

pub const MASK_OP: u16 = 0b11111;
pub const SHIFT_A: u16 = 10;
pub const SHIFT_B: u16 = 5;
pub const MASK_B: u16 = 0b11111;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Instruction<I> {
    /// op b a
    BasicOp(BasicOp, Value<I>, Value<I>),
    SpecialOp(SpecialOp, Value<I>)
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
