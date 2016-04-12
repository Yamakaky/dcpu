use std::str;
use std::str::FromStr;

use nom::*;

use types::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedItem {
    Label(String),
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
    Num(u16),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>)
}

fn bytes_to_type<I: FromStr>(i: &[u8]) -> Result<I, ()> {
    str::from_utf8(i)
        .map_err(|_| ())
        .and_then(|x| FromStr::from_str(x).map_err(|_| ()))
        .map_err(|_| ())
}

named!(hex_num(&[u8]) -> &[u8],
    recognize!(chain!(tag!("0x") ~ n: many1!(hex_digit), || ()))
);

named!(octal_num(&[u8]) -> &[u8],
    recognize!(chain!(tag!("0o") ~ many1!(one_of!("01234567")), || ()))
);

named!(bin_num(&[u8]) -> &[u8],
    recognize!(chain!(tag!("0b") ~ many1!(one_of!("01")), || ()))
);

named!(num(&[u8]) -> &[u8],
    recognize!(many1!(digit))
);

named!(number(&[u8]) -> u16,
    map_res!(
         recognize!(alt_complete!(hex_num | octal_num | bin_num | num)),
         bytes_to_type
     )
);

named!(comment(&[u8]) -> ParsedItem,
    map!(
        map_res!(
            delimited!(tag!(";"), not_line_ending, line_ending),
            bytes_to_type
        ),
        ParsedItem::Comment
    )
);

named!(basic_op(&[u8]) -> BasicOp,
    map_res!(
        take!(3),
        bytes_to_type
    )
);

named!(instruction(&[u8]) -> ParsedInstruction,
    alt_complete!(basic_instruction | special_instruction)
);

named!(basic_instruction(&[u8]) -> ParsedInstruction,
    chain!(
        op: basic_op ~
        many1!(space) ~
        b: b_value ~
        many0!(space) ~
        char!(',') ~
        many0!(space) ~
        a: a_value,

        || ParsedInstruction::BasicOp(op, b, a)
    )
);

named!(special_op(&[u8]) -> SpecialOp,
    map_res!(
        take!(3),
        bytes_to_type
    )
);

named!(special_instruction(&[u8]) -> ParsedInstruction,
    chain!(
        op: special_op ~
        many1!(space) ~
        a: a_value,

        || ParsedInstruction::SpecialOp(op, a)
    )
);

named!(register(&[u8]) -> Register,
    map_res!(
        alpha,
        bytes_to_type
    )
);

named!(at_reg_plus(&[u8]) -> ParsedValue,
    chain!(
        char!('[') ~
        reg: register ~
        char!('+') ~
        e: expression ~
        char!(']'),
        || ParsedValue::AtRegPlus(reg, e)
    )
);

named!(value(&[u8]) -> ParsedValue,
    alt_complete!(
        map!(register, ParsedValue::Reg) |
        map!(delimited!(char!('['), register, char!(']')),
             ParsedValue::AtReg) |
        map!(delimited!(char!('['), expression, char!(']')),
             ParsedValue::AtAddr) |
        at_reg_plus |
        map!(
            chain!(
                tag!("PICK ") ~
                n: expression,
                || n
            ),
            ParsedValue::Pick
        ) |
        map!(tag!("SP"), |_| ParsedValue::SP) |
        map!(tag!("PC"), |_| ParsedValue::PC) |
        map!(tag!("EX"), |_| ParsedValue::EX)
    )
);

named!(label(&[u8]) -> String,
    map_res!(
        recognize!(preceded!(
            alt_complete!(alpha | tag!("_")),
            many0!(alt_complete!(alphanumeric | tag!("_")))
            )
        ),
        bytes_to_type
    )
);

named!(expression(&[u8]) -> Expression,
    alt_complete!(
        map!(number, Expression::Num) |
        map!(label, Expression::Label)
    )
);

named!(a_value(&[u8]) -> ParsedValue,
    alt_complete!(
        value |
        map!(expression, ParsedValue::Litteral) |
        map!(tag!("POP"), |_| ParsedValue::Push)
    )
);

named!(b_value(&[u8]) -> ParsedValue,
    alt_complete!(
        value |
        map!(expression, ParsedValue::Litteral) |
        map!(tag!("PUSH"), |_| ParsedValue::Push)
    )
);

named!(pub parse(&[u8]) -> Vec<ParsedItem>,
    complete!(separated_list!(line_ending,
                              map!(instruction,
                                   ParsedItem::ParsedInstruction)))
);


#[cfg(test)]
#[test]
fn test_num() {
    let empty: &[u8] = &[];
    assert_eq!(number("1".as_bytes()), IResult::Done(empty, 1));
    //assert_eq!(number("0b1".as_bytes()), IResult::Done(empty, 1));
    //assert_eq!(number("0x1".as_bytes()), IResult::Done(empty, 1));
    //assert_eq!(number("0o1".as_bytes()), IResult::Done(empty, 1));
}

#[cfg(test)]
#[test]
fn test_instruction() {
    let empty: &[u8] = &[];
    assert_eq!(instruction("ADD A, B".as_bytes()),
               IResult::Done(empty,
                             ParsedInstruction::BasicOp(BasicOp::ADD,
                                                        ParsedValue::Reg(Register::A),
                                                        ParsedValue::Reg(Register::B))));
}

#[cfg(test)]
#[test]
fn test_register() {
    let empty: &[u8] = &[];
    assert_eq!(register("A".as_bytes()),
               IResult::Done(empty, Register::A));
}

#[cfg(test)]
#[test]
fn test_basic_op() {
    let empty: &[u8] = &[];
    assert_eq!(basic_op("ADD".as_bytes()),
               IResult::Done(empty, BasicOp::ADD));
}
