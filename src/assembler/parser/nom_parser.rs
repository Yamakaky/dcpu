use std::str::{self, FromStr};

use nom::*;

use assembler::types::*;

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

named!(pub pos_number<u16>,
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

named!(instruction<Instruction<Expression> >,
    alt_complete!(basic_instruction | special_instruction)
);

named!(basic_instruction<Instruction<Expression> >,
    chain!(
        op: basic_op ~
        multispace ~
        b: b_value ~
        multispace? ~
        char!(',') ~
        multispace? ~
        a: a_value,

        || Instruction::BasicOp(op, b, a)
    )
);

named!(special_op<SpecialOp>,
    map_res!(
        take!(3),
        bytes_to_type
    )
);

named!(special_instruction<Instruction<Expression> >,
    chain!(
        op: special_op ~
        multispace ~
        a: a_value,

        || Instruction::SpecialOp(op, a)
    )
);

named!(register<Register>,
    map_res!(
        alpha,
        bytes_to_type
    )
);

named!(at_reg_plus<Value<Expression> >,
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
        || Value::AtRegPlus(reg, e)
    )
);

named!(value<Value<Expression> >,
    alt_complete!(
        map!(register, Value::Reg) |
        at_reg_plus |
        map!(chain!(char!('[') ~
                    multispace? ~
                    r: register ~
                    multispace? ~
                    char!(']'),
                    || r),
             Value::AtReg) |
        map!(chain!(char!('[') ~
                    multispace? ~
                    e: expression ~
                    multispace? ~
                    char!(']'),
                    || e),
             Value::AtAddr) |
        map!(
            chain!(
                tag!("PICK") ~
                space ~
                n: expression,
                || n
            ),
            Value::Pick
        ) |
        map!(tag!("PEEK"), |_| Value::Peek) |
        map!(tag!("SP"), |_| Value::SP) |
        map!(tag!("PC"), |_| Value::PC) |
        map!(tag!("EX"), |_| Value::EX) |
        map!(expression, Value::Litteral)
    )
);

named!(raw_label<String>,
    map_res!(
        recognize!(
            preceded!(
                alt_complete!(alpha | tag!("_") | tag!(".")),
                many0!(alt_complete!(alphanumeric | tag!("_") | tag!(".")))
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
        char!(':') ~
        l: raw_label,
        || ParsedItem::LabelDecl(l)
    )
);

named!(local_label_decl<ParsedItem>,
    chain!(
        char!(':') ~
        l: raw_local_label,
        || ParsedItem::LocalLabelDecl(l)
    )
);

named!(simple_expression<Expression>,
    alt_complete!(
        map!(number, Expression::Num) |
        map!(raw_local_label, Expression::LocalLabel) |
        map!(raw_label, Expression::Label)
    )
);

named!(pub expression<Expression>,
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
        chain!(e1: simple_expression ~
               multispace? ~
               char!('<') ~
               multispace? ~
               e2: expression,
               || Expression::Less(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!("<=") ~
               multispace? ~
               e2: expression,
               || Expression::Not(
                     Box::new(Expression::Greater(Box::new(e1),
                                                  Box::new(e2)))))  |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!("==") ~
               multispace? ~
               e2: expression,
               || Expression::Equal(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!("!=") ~
               multispace? ~
               e2: expression,
               || Expression::Not(
                      Box::new(Expression::Equal(Box::new(e1),
                                                 Box::new(e2))))) |
        chain!(e1: simple_expression ~
               multispace? ~
               char!('>') ~
               multispace? ~
               e2: expression,
               || Expression::Greater(Box::new(e1), Box::new(e2))) |
        chain!(e1: simple_expression ~
               multispace? ~
               tag!(">=") ~
               multispace? ~
               e2: expression,
               || Expression::Not(
                     Box::new(Expression::Less(Box::new(e1),
                                               Box::new(e2))))) |
        chain!(char!('!') ~
               multispace? ~
               e: expression,
               || Expression::Not(Box::new(e))) |
        simple_expression
    )
);

named!(a_value<Value<Expression> >,
    alt_complete!(
        map!(tag!("POP"), |_| Value::Push) |
        value
    )
);

named!(b_value<Value<Expression> >,
    alt_complete!(
        map!(tag!("PUSH"), |_| Value::Push) |
        value
    )
);

named!(string<String>,
    map_res!(
        delimited!(tag!("\""), recognize!(many0!(none_of!("\""))), tag!("\"")),
        bytes_to_type
    )
);

named!(dir_dat<Directive>,
    chain!(alt_complete!(tag!("dat") | tag!("byte")| tag!("word") | tag!("short")) ~
           space ~
           ns: separated_list!(space,
                               alt_complete!(map!(expression, From::from) |
                                             map!(string, From::from))),
           || Directive::Dat(ns))
);

named!(dir_org<Directive>,
    chain!(tag!("org") ~
           space ~
           n: number ~
           tag!(",")? ~
           space ~
           val: number,
           || Directive::Org(n.into(), val.into()))
);

named!(dir_skip<Directive>,
    chain!(tag!("skip") ~
           space ~
           n: number ~
           tag!(",")? ~
           space ~
           val: number,
           || Directive::Skip(n.into(), val.into()))
);

named!(dir_zero<Directive>,
    chain!(tag!("zero") ~
           space ~
           n: number,
           || Directive::Skip(n.into(), 0))
);

named!(dir_global<Directive>,
    chain!(tag!("globl") ~
           many0!(none_of!("\n")),
           || Directive::Global)
);

named!(dir_text<Directive>,
    chain!(tag!("text") ~
           many0!(none_of!("\n")),
           || Directive::Text)
);

named!(dir_bss<Directive>,
    chain!(tag!("bss") ~
           many0!(none_of!("\n")),
           || Directive::BSS)
);

named!(dir_lcomm<Directive>,
    chain!(alt_complete!(tag!("lcomm") | tag!("comm")) ~
           space ~
           symbol: raw_label ~
           tag!(",")? ~
           space? ~
           size: number,
           || Directive::Lcomm(symbol, size.into()))
);

named!(directive<Directive>,
    chain!(char!('.') ~
           d: alt_complete!(dir_dat |
                            dir_org |
                            dir_skip |
                            dir_zero |
                            dir_global |
                            dir_text |
                            dir_lcomm |
                            dir_bss) ~
           peek!(line_ending),
           || d)
);

named!(pub parse< Vec<ParsedItem> >,
    delimited!(
        opt!(multispace),
        separated_list!(multispace,
                        alt_complete!(
                            map!(directive, ParsedItem::Directive) |
                            map!(instruction,
                                 ParsedItem::Instruction) |
                            comment |
                            local_label_decl |
                            label_decl
                        )
        ),
        opt!(multispace)
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
                             Instruction::BasicOp(BasicOp::ADD,
                                                  Value::Reg(Register::A),
                                                  Value::Reg(Register::B))));
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
                             Directive::Dat(vec!(DatItem::E(Expression::Num(Num::U(1))),
                                                 DatItem::E(Expression::Num(Num::U(2)))))));
}
