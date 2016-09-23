use emulator::cpu::Cpu;
use emulator::device::*;

#[derive(Debug)]
pub struct Dummy {
    hardware_id: u32,
    hardware_version: u16,
    manufacturer: u32,
    interrupt_delay: u16,
}

impl Device for Dummy {
    fn hardware_id(&self) -> u32 {
        self.hardware_id
    }

    fn hardware_version(&self) -> u16 {
        self.hardware_version
    }

    fn manufacturer(&self) -> u32 {
        self.manufacturer
    }

    fn interrupt(&mut self, _: &mut Cpu) -> InterruptResult {
        Ok(self.interrupt_delay)
    }

    fn tick(&mut self, _: &mut Cpu, _: u64) -> TickResult {
        TickResult::Nothing
    }

    fn inspect(&self) {
        println!("Dummy device");
    }
}
