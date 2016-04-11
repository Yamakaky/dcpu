#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate num;
extern crate simplelog;

mod parser;
mod types;
mod dcpu;

use types::*;

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Trace).unwrap();

    let ast = Ast {
        instructions: vec![
            Instruction::BasicOp(BasicOp::ADD,
                                 Value::Reg(Register::A),
                                 Value::AtAddr(10)),
            Instruction::BasicOp(BasicOp::SET,
                                 Value::Reg(Register::B),
                                 Value::Reg(Register::A))
        ]
    };
    println!("{}", ast);

    let mut dcpu: dcpu::Dcpu = Default::default();
    dcpu.load_ops(&ast.instructions, 0);
    dcpu.load(&[9], 10);
    dcpu.tick().unwrap();
    trace!("{:?}", dcpu.registers);
    dcpu.tick().unwrap();
    dcpu.tick().unwrap();
    dcpu.tick().unwrap();
    dcpu.tick().unwrap();
}
