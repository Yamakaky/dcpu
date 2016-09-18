extern crate byteorder;
extern crate dcpu;

mod utils;

fn main() {
    let ram: Vec<u16> = utils::IterU16 {
        input: utils::get_input(Some("a".into()))
    }.collect();
    let mut cpu = dcpu::cpu::Cpu::default();
    cpu.ram[..ram.len()].clone_from_slice(&ram);
    let mut debugger = dcpu::debugger::Debugger::new(cpu);
    debugger.run();
}
