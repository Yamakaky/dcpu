extern crate dcpu;
extern crate docopt;
#[macro_use]
extern crate log;
extern crate simplelog;

use dcpu::cpu::Cpu;

fn main() {
    simplelog::TermLogger::init(simplelog::LogLevelFilter::Trace).unwrap();

    let cpu = Cpu::default();
}
