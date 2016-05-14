use std::collections::HashMap;
use types::{BasicOp, SpecialOp, Register, Value, Instruction};

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
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedInstruction {
    BasicOp(BasicOp, ParsedValue, ParsedValue),
    SpecialOp(SpecialOp, ParsedValue),
}

impl ParsedInstruction {
    fn solve(&self,
             globals: &HashMap<String, u16>,
             locals: &HashMap<String, u16>)
             -> Result<Instruction, String> {
        match *self {
            ParsedInstruction::BasicOp(op, ref b, ref a) => {
                Ok(Instruction::BasicOp(op,
                                        try!(b.solve(globals, locals)),
                                        try!(a.solve(globals, locals))))
            }
            ParsedInstruction::SpecialOp(op, ref a) => {
                Ok(Instruction::SpecialOp(op, try!(a.solve(globals, locals))))
            }
        }
    }
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
    Litteral(Expression),
}

impl ParsedValue {
    fn solve(&self,
             globals: &HashMap<String, u16>,
             locals: &HashMap<String, u16>)
             -> Result<Value, String> {
        match *self {
            ParsedValue::Reg(r) => Ok(Value::Reg(r)),
            ParsedValue::AtReg(r) => Ok(Value::AtReg(r)),
            ParsedValue::AtRegPlus(r, ref e) => {
                Ok(Value::AtRegPlus(r, try!(e.solve(globals, locals))))
            }
            ParsedValue::Push => Ok(Value::Push),
            ParsedValue::Peek => Ok(Value::Peek),
            ParsedValue::Pick(ref e) => Ok(Value::Pick(try!(e.solve(globals, locals)))),
            ParsedValue::SP => Ok(Value::SP),
            ParsedValue::PC => Ok(Value::PC),
            ParsedValue::EX => Ok(Value::EX),
            ParsedValue::AtAddr(ref e) => Ok(Value::AtAddr(try!(e.solve(globals, locals)))),
            ParsedValue::Litteral(ref e) => Ok(Value::Litteral(try!(e.solve(globals, locals)))),
        }
    }
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
    Mod(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn solve(&self,
             globals: &HashMap<String, u16>,
             locals: &HashMap<String, u16>)
             -> Result<u16, String> {
        match *self {
            Expression::Label(ref s) => {
                match globals.get(s) {
                    Some(addr) => Ok(*addr),
                    None => Err(s.clone()),
                }
            }
            Expression::LocalLabel(ref s) => {
                match locals.get(s) {
                    Some(addr) => Ok(*addr),
                    None => Err(s.clone()),
                }
            }
            Expression::Num(n) => Ok(n.into()),
            Expression::Add(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)).wrapping_add(try!(r.solve(globals, locals))))
            }
            Expression::Sub(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)).wrapping_sub(try!(r.solve(globals, locals))))
            }
            Expression::Mul(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)).wrapping_mul(try!(r.solve(globals, locals))))
            }
            Expression::Div(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)).wrapping_div(try!(r.solve(globals, locals))))
            }
            Expression::Shr(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)) >> try!(r.solve(globals, locals)))
            }
            Expression::Shl(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)) << try!(r.solve(globals, locals)))
            }
            Expression::Mod(ref l, ref r) => {
                Ok(try!(l.solve(globals, locals)) % try!(r.solve(globals, locals)))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Num {
    U(u16),
    I(i16),
}

impl From<Num> for u16 {
    fn from(n: Num) -> u16 {
        match n {
            Num::U(u) => u,
            Num::I(i) => i as u16,
        }
    }
}

impl From<Num> for Expression {
    fn from(n: Num) -> Expression {
        Expression::Num(n)
    }
}
