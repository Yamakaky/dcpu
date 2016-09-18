use std::collections::HashMap;
use std::iter;

use types::{BasicOp, SpecialOp, Register, Value, Instruction};
use assembler::linker::{Error, Globals, Locals};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    Dat(Vec<DatItem>),
    Org(u16, u16),
    Skip(u16, u16),
    Global,
    Text,
    BSS,
    /// Symbol, size
    Lcomm(String, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatItem {
    S(String),
    E(Expression),
}

impl Directive {
    pub fn append_to(&self, bin: &mut Vec<u16>, globals: &Globals, locals: &Locals) -> Result<u16, Error> {
        match *self {
            Directive::Dat(ref v) => {
                let mut i = 0;
                for x in v.iter() {
                    i += match *x {
                        DatItem::S(ref s) => {
                            let it = s.bytes().chain(iter::once(0));
                            let size = it.size_hint().0;
                            assert!(size == it.size_hint().1.unwrap());
                            bin.extend(it.map(|x| x as u16));
                            size
                        }
                        DatItem::E(ref e) => {
                            bin.push(try!(e.solve(globals, locals)));
                            1
                        }
                    }
                }
                Ok(i as u16)
            }
            Directive::Org(n, val) => {
                assert!(n as usize > bin.len(),
                        "`.org` can't be used to go backward: current = {}, n = {}",
                        bin.len() - 1,
                        n);
                bin.resize((n as usize), val);
                Ok(n)
            }
            Directive::Skip(n, val) => {
                let l = bin.len();
                bin.resize(l + (n as usize), val);
                Ok(n)
            }
            Directive::Global | Directive::Text | Directive::BSS => Ok(0),
            Directive::Lcomm(_, _) => unreachable!(),
        }
    }
}

impl From<String> for DatItem {
    fn from(s: String) -> DatItem {
        DatItem::S(s)
    }
}

impl From<Expression> for DatItem {
    fn from(e: Expression) -> DatItem {
        DatItem::E(e)
    }
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
    pub fn solve(&self,
                 globals: &Globals,
                 locals: &Locals)
                 -> Result<Instruction, Error> {
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
             -> Result<Value, Error> {
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
    Not(Box<Expression>),
    /// a < b
    Less(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    /// a > b
    Greater(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn solve(&self,
             globals: &HashMap<String, u16>,
             locals: &HashMap<String, u16>)
             -> Result<u16, Error> {
        match *self {
            Expression::Label(ref s) => {
                match globals.get(s) {
                    Some(addr) => Ok(*addr),
                    None => Err(Error::UnknownLabel(s.clone())),
                }
            }
            Expression::LocalLabel(ref s) => {
                match locals.get(s) {
                    Some(addr) => Ok(*addr),
                    None => Err(Error::UnknownLocalLabel(s.clone())),
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
            Expression::Not(ref l) => {
                Ok(if try!(l.solve(globals, locals)) == 0 {1} else {0})
            }
            Expression::Less(ref l, ref r) => {
                Ok((try!(l.solve(globals, locals)) <
                    try!(r.solve(globals, locals))) as u16)
            }
            Expression::Equal(ref l, ref r) => {
                Ok((try!(l.solve(globals, locals)) ==
                    try!(r.solve(globals, locals))) as u16)
            }
            Expression::Greater(ref l, ref r) => {
                Ok((try!(l.solve(globals, locals)) >
                    try!(r.solve(globals, locals))) as u16)
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
