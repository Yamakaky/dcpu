extern crate dcpu;

fn main() {
    let mut debugger = dcpu::debugger::Debugger::new();
    debugger.run();
}
