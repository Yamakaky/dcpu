use types::{BasicOp, SpecialOp, Register};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    Dat(Vec<u16>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedItem {
    Directive(Directive),
    LabelDecl(String),
    LocalLabelDecl(String),
    ParsedInstruction(ParsedInstruction),
    Comment(String)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedInstruction {
    BasicOp(BasicOp, ParsedValue, ParsedValue),
    SpecialOp(SpecialOp, ParsedValue)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedValue {
    Reg(Register),
    AtReg(Register),
    AtRegPlus(Register, Expression),
    Push,
    Peek,
    Pick(Expression),
    SP,
    PC,
    EX,
    AtAddr(Expression),
    Litteral(Expression)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Label(String),
    LocalLabel(String),
    Num(Num),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Shr(Box<Expression>, Box<Expression>),
    Shl(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Num {
    U(u16),
    I(i16)
}

impl From<Num> for u16 {
    fn from(n: Num) -> u16 {
        match n {
            Num::U(u) => u,
            Num::I(i) => i as u16
        }
    }
}

impl From<Num> for Expression {
    fn from(n: Num) -> Expression {
        Expression::Num(n)
    }
}

