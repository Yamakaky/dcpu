use std::collections::HashMap;
use std::fmt;
use std::iter;

pub use types::{BasicOp, SpecialOp, Register, Value, Instruction};
use assembler::linker::*;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LabelInfos {
    pub addr: u16,
    pub locals: HashMap<String, u16>,
}
pub type Globals = HashMap<String, LabelInfos>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedItem {
    Directive(Directive),
    LabelDecl(String),
    LocalLabelDecl(String),
    Instruction(Instruction<Expression>),
    Comment(String),
}

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

impl Directive {
    pub fn append_to(&self,
                     bin: &mut Vec<u16>,
                     labels: &Globals,
                     last_global: &Option<String>) -> Result<u16> {
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
                            bin.push(try!(e.solve(labels, last_global)));
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatItem {
    S(String),
    E(Expression),
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

impl Instruction<Expression> {
    pub fn solve(&self, globals: &Globals, last_global: &Option<String>)
        -> Result<Instruction<u16>> {
        match *self {
            Instruction::BasicOp(op, ref b, ref a) => {
                Ok(Instruction::BasicOp(op,
                                        try!(b.solve(globals, last_global)),
                                        try!(a.solve(globals, last_global))))
            }
            Instruction::SpecialOp(op, ref a) => {
                Ok(Instruction::SpecialOp(op,
                                          try!(a.solve(globals, last_global))))
            }
        }
    }
}

impl Value<Expression> {
    fn solve(&self, globals: &Globals, last_global: &Option<String>)
             -> Result<Value<u16>> {
        match *self {
            Value::Reg(r) => Ok(Value::Reg(r)),
            Value::AtReg(r) => Ok(Value::AtReg(r)),
            Value::AtRegPlus(r, ref e) =>
                Ok(Value::AtRegPlus(r, try!(e.solve(globals, last_global)))),
            Value::Push => Ok(Value::Push),
            Value::Peek => Ok(Value::Peek),
            Value::Pick(ref e) =>
                Ok(Value::Pick(try!(e.solve(globals, last_global)))),
            Value::SP => Ok(Value::SP),
            Value::PC => Ok(Value::PC),
            Value::EX => Ok(Value::EX),
            Value::AtAddr(ref e) =>
                Ok(Value::AtAddr(try!(e.solve(globals, last_global)))),
            Value::Litteral(ref e) =>
                Ok(Value::Litteral(try!(e.solve(globals, last_global)))),
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
    pub fn solve(&self, globals: &Globals, last_global: &Option<String>)
        -> Result<u16> {
        match *self {
            Expression::Label(ref s) => {
                match globals.get(s) {
                    Some(i) => Ok(i.addr),
                    None => try!(Err(ErrorKind::UnknownLabel(s.clone()))),
                }
            }
            Expression::LocalLabel(ref s) => {
                match globals.get(last_global.as_ref()
                                             .unwrap())
                             .unwrap()
                             .locals
                             .get(s) {
                    Some(addr) => Ok(*addr),
                    None => try!(Err(ErrorKind::UnknownLocalLabel(s.clone()))),
                }
            }
            Expression::Num(n) => Ok(n.into()),
            Expression::Add(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)).wrapping_add(try!(r.solve(globals, last_global))))
            }
            Expression::Sub(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)).wrapping_sub(try!(r.solve(globals, last_global))))
            }
            Expression::Mul(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)).wrapping_mul(try!(r.solve(globals, last_global))))
            }
            Expression::Div(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)).wrapping_div(try!(r.solve(globals, last_global))))
            }
            Expression::Shr(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)) >> try!(r.solve(globals, last_global)))
            }
            Expression::Shl(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)) << try!(r.solve(globals, last_global)))
            }
            Expression::Mod(ref l, ref r) => {
                Ok(try!(l.solve(globals, last_global)) % try!(r.solve(globals, last_global)))
            }
            Expression::Not(ref l) => {
                Ok(if try!(l.solve(globals, last_global)) == 0 {1} else {0})
            }
            Expression::Less(ref l, ref r) => {
                Ok((try!(l.solve(globals, last_global)) <
                    try!(r.solve(globals, last_global))) as u16)
            }
            Expression::Equal(ref l, ref r) => {
                Ok((try!(l.solve(globals, last_global)) ==
                    try!(r.solve(globals, last_global))) as u16)
            }
            Expression::Greater(ref l, ref r) => {
                Ok((try!(l.solve(globals, last_global)) >
                    try!(r.solve(globals, last_global))) as u16)
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Label(ref s) => write!(f, "{}", s),
            Expression::LocalLabel(ref s) => write!(f, ".{}", s),
            Expression::Num(ref n) => write!(f, "0x{:0>4x}", u16::from(*n)),
            Expression::Add(ref l, ref r) => write!(f, "{} + {}", l, r),
            Expression::Sub(ref l, ref r) => write!(f, "{} - {}", l, r),
            Expression::Mul(ref l, ref r) => write!(f, "{} * {}", l, r),
            Expression::Div(ref l, ref r) => write!(f, "{} / {}", l, r),
            Expression::Shr(ref l, ref r) => write!(f, "{} >> {}", l, r),
            Expression::Shl(ref l, ref r) => write!(f, "{} << {}", l, r),
            Expression::Mod(ref l, ref r) => write!(f, "{} % {}", l, r),
            Expression::Not(ref e) => write!(f, "!{}", e),
            Expression::Less(ref l, ref r) => write!(f, "{} < {}", l, r),
            Expression::Equal(ref l, ref r) => write!(f, "{} == {}", l, r),
            Expression::Greater(ref l, ref r) => write!(f, "{} > {}", l, r),
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
