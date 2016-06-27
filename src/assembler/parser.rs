use std::str;
use std::str::FromStr;

use nom::*;

use assembler::types::*;
use types::{BasicOp, SpecialOp, Register};

fn bytes_to_type<I: FromStr>(i: &[u8]) -> Result<I, ()> {
    str::from_utf8(i)
        .map_err(|_| ())
        .and_then(|x| FromStr::from_str(x).map_err(|_| ()))
        .map_err(|_| ())
}

named!(hex_num<(&str, u32)>,
    map_res!(chain!(tag!("0x") ~ n: recognize!(many1!(hex_digit)), || n),
             |n| str::from_utf8(n).map(|n| (n, 16)))
);

named!(num<(&str, u32)>,
    map_res!(chain!(n: recognize!(many1!(digit)), || n),
             |n| str::from_utf8(n).map(|n| (n, 10)))
);

named!(octal_num<(&str, u32)>,
    map_res!(chain!(tag!("0o") ~ n: recognize!(many1!(one_of!("01234567"))), || n),
             |n| str::from_utf8(n).map(|n| (n, 8)))
);

named!(bin_num<(&str, u32)>,
    map_res!(chain!(tag!("0b") ~ n: recognize!(many1!(one_of!("01"))), || n),
             |n| str::from_utf8(n).map(|n| (n, 2)))
);

named!(pos_number<u16>,
    map_res!(
        alt_complete!(hex_num | octal_num | bin_num | num),
        |(n, base)| u16::from_str_radix(n, base)
    )
);

named!(neg_number<i16>,
    map_res!(
        chain!(char!('-') ~
               n: alt_complete!(hex_num | octal_num | bin_num | num),
               || n),
        |(n, base)| i16::from_str_radix(&format!("-{}", n), base)
    )
);

named!(number<Num>,
    alt_complete!(map!(neg_number, Num::I) |
                  map!(pos_number, Num::U))
);

named!(comment<ParsedItem>,
    map!(
        map_res!(
            delimited!(tag!(";"), not_line_ending, peek!(line_ending)),
            bytes_to_type
        ),
        ParsedItem::Comment
    )
);

named!(basic_op<BasicOp>,
    map_res!(
        take!(3),
        bytes_to_type
    )
);

named!(instruction<ParsedInstruction>,
    alt_complete!(basic_instruction | special_instruction)
);

named!(basic_instruction<ParsedInstruction>,
    chain!(
        op: basic_op ~
        multispace ~
        b: b_value ~
        multispace? ~
        char!(',') ~
        multispace? ~
        a: a_value,

        || ParsedInstruction::BasicOp(op, b, a)
    )
);

named!(special_op<SpecialOp>,
    map_res!(
        take!(3),
        bytes_to_type
    )
);

named!(special_instruction<ParsedInstruction>,
    chain!(
        op: special_op ~
        multispace ~
        a: a_value,

        || ParsedInstruction::SpecialOp(op, a)
    )
);

named!(register<Register>,
    map_res!(
        alpha,
        bytes_to_type
    )
);

named!(at_reg_plus<ParsedValue>,
    chain!(
        char!('[') ~
        multispace? ~
        reg: register ~
        multispace? ~
        char!('+') ~
        multispace? ~
        e: expression ~
        multispace? ~
        char!(']'),
        || ParsedValue::AtRegPlus(reg, e)
    )
);

named!(value<ParsedValue>,
    alt_complete!(
        map!(register, ParsedValue::Reg) |
        map!(chain!(char!('[') ~
                    multispace? ~
                    r: register ~
                    multispace? ~
                    char!(']'),
                    || r),
             ParsedValue::AtReg) |
        map!(chain!(char!('[') ~
                    multispace? ~
                    e: expression ~
                    multispace? ~
                    char!(']'),
                    || e),
             ParsedValue::AtAddr) |
        at_reg_plus |
        map!(
            chain!(
                tag!("PICK") ~
                space ~
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

named!(raw_label<String>,
    map_res!(
        recognize!(
            preceded!(
                alt_complete!(alpha | tag!("_")),
                many0!(alt_complete!(alphanumeric | tag!("_")))
            )
        ),
        bytes_to_type
    )
);

named!(raw_local_label<String>,
    chain!(char!('.') ~ l: raw_label, || l)
);

named!(label_decl<ParsedItem>,
    chain!(
        opt!(char!(':')) ~
        l: raw_label ~
        opt!(char!(':')),
        || ParsedItem::LabelDecl(l)
    )
);

named!(local_label_decl<ParsedItem>,
    chain!(
        opt!(char!(':')) ~
        l: raw_local_label ~
        opt!(char!(':')),
        || ParsedItem::LocalLabelDecl(l)
    )
);

named!(simple_expression<Expression>,
    alt_complete!(
        map!(number, Expression::Num) |
        map!(raw_label, Expression::Label) |
        map!(raw_local_label, Expression::LocalLabel)
    )
);

named!(expression<Expression>,
    alt_complete!(
        chain!(char!('(') ~
               multispace? ~
               e: expression ~
               multispace? ~
               char!(')'),
               || e) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('+') ~
               multispace? ~
               e2: expression,
               || Expression::Add(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('-') ~
               multispace? ~
               e2: expression,
               || Expression::Sub(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('*') ~
               multispace? ~
               e2: expression,
               || Expression::Mul(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('/') ~
               multispace? ~
               e2: expression,
               || Expression::Div(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!(">>") ~
               multispace? ~
               e2: expression,
               || Expression::Shr(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!("<<") ~
               multispace? ~
               e2: expression,
               || Expression::Shl(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('%') ~
               multispace? ~
               e2: expression,
               || Expression::Mod(Box::new(e1), Box::new(e2))) |
        simple_expression
    )
);

named!(a_value<ParsedValue>,
    alt_complete!(
        value |
        map!(expression, ParsedValue::Litteral) |
        map!(tag!("POP"), |_| ParsedValue::Push)
    )
);

named!(b_value<ParsedValue>,
    alt_complete!(
        value |
        map!(expression, ParsedValue::Litteral) |
        map!(tag!("PUSH"), |_| ParsedValue::Push)
    )
);

named!(dir_dat<Directive>,
    chain!(tag!("dat") ~
           space ~
           ns: separated_list!(space,
                               number),
           || Directive::Dat(ns.into_iter().map(From::from).collect()))
);

named!(dir_org<Directive>,
    chain!(tag!("org") ~
           space ~
           n: number,
           || Directive::Org(n.into()))
);

named!(directive<Directive>,
    chain!(char!('.') ~
           d: alt_complete!(dir_dat | dir_org) ~
           peek!(line_ending),
           || d)
);

named!(pub parse< Vec<ParsedItem> >,
    separated_list!(multispace,
                    alt_complete!(
                        map!(directive, ParsedItem::Directive) |
                        map!(instruction,
                             ParsedItem::ParsedInstruction) |
                        comment |
                        label_decl |
                        local_label_decl)
    )
);


#[cfg(test)]
const EMPTY: &'static [u8] = &[];

#[cfg(test)]
#[test]
fn test_num() {
    assert_eq!(number("1".as_bytes()), IResult::Done(EMPTY, Num::U(1)));
    assert_eq!(number("0b1".as_bytes()), IResult::Done(EMPTY, Num::U(1)));
    assert_eq!(number("0x1".as_bytes()), IResult::Done(EMPTY, Num::U(1)));
    assert_eq!(number("0o1".as_bytes()), IResult::Done(EMPTY, Num::U(1)));
    assert_eq!(number("-0o1".as_bytes()), IResult::Done(EMPTY, Num::I(-1)));
}

#[cfg(test)]
#[test]
fn test_instruction() {
    assert_eq!(instruction("ADD A, B".as_bytes()),
               IResult::Done(EMPTY,
                             ParsedInstruction::BasicOp(BasicOp::ADD,
                                                        ParsedValue::Reg(Register::A),
                                                        ParsedValue::Reg(Register::B))));
}

#[cfg(test)]
#[test]
fn test_register() {
    assert_eq!(register("A".as_bytes()),
               IResult::Done(EMPTY, Register::A));
}

#[cfg(test)]
#[test]
fn test_basic_op() {
    assert_eq!(basic_op("ADD".as_bytes()),
               IResult::Done(EMPTY, BasicOp::ADD));
}

#[cfg(test)]
#[test]
fn test_expression() {
    assert_eq!(expression("1 + 2".as_bytes()),
               IResult::Done(EMPTY,
                             Expression::Add(Box::new(Expression::Num(Num::U(1))),
                                             Box::new(Expression::Num(Num::U(2))))));
    assert_eq!(expression("1-2".as_bytes()),
               IResult::Done(EMPTY,
                             Expression::Sub(Box::new(Expression::Num(Num::U(1))),
                                             Box::new(Expression::Num(Num::U(2))))));
    assert_eq!(expression("(1)".as_bytes()),
               IResult::Done(EMPTY,
                             Expression::Num(Num::U(1))));
}

#[cfg(test)]
#[test]
fn test_directive() {
    let nl: &[u8] = &[10];
    assert_eq!(directive(".dat 1 0x2\n".as_bytes()),
               IResult::Done(nl,
                             Directive::Dat(vec!(1, 2))));
}
