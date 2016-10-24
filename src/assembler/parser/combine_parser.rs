use combine::*;
use combine::char::*;

use assembler::types::*;

type RN<'a, O> = Box<Parser<Input=&'a str, Output=O>>;

fn raw_number() -> RN<(&str, usize)> {
    let hex_num = (string("0x"), many1(hex_digit())).map(|(_, digits)| {
        (digits, 16)
    });
    let num = many1(hex_digit()).map(|digits| {
        (digits, 10)
    });
    let octal_num = (string("0o"), many1(hex_digit())).map(|(_, digits)| {
        (digits, 8)
    });
    let bin_num = (string("0b"), many1(hex_digit())).map(|(_, digits)| {
        (digits, 2)
    });
    hex_num.or(octal_num).or(bin_num).or(num)
}

type PN = u16;

fn pos_number() -> PN {
    raw_number().and_then(|(digits, base)| u16::from_str_radix(digits,
                                                                    base))
}

type SN = u16;

fn signed_number() -> SN {
    (char('-'), raw_number()).and_then(|(_, (digits, base))| {
        i16::from_str_radix(digits, base)
    })
}

fn number(input: &str) -> ParseResult<Num, &str> {
    signed_number().map(Num::I).or(pos_number().map(Num::U))
}
