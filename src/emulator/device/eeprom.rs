use std::any::Any;

use emulator::Cpu;
use emulator::device::*;
use types::Register;

const ONE_MS: u16 = 100;
const MEMORY_SIZE: usize = 16;

#[derive(Debug)]
pub struct Eeprom<D: Device> {
    mem: [u16; MEMORY_SIZE],
    inner: D,
}

impl<D: Device> Eeprom<D> {
    pub fn new(inner: D) -> Eeprom<D> {
        Eeprom {
            mem: [0; MEMORY_SIZE],
            inner: inner,
        }
    }
}

impl<D: Device> Device for Eeprom<D> {
    fn hardware_id(&self) -> u32 {
        self.inner.hardware_id()
    }

    fn hardware_version(&self) -> u16 {
        self.inner.hardware_version()
    }

    fn manufacturer(&self) -> u32 {
        self.inner.manufacturer()
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay> {
        let a = cpu.registers[Register::A];
        let b = cpu.registers[Register::B];
        let x = cpu.registers[Register::X] as usize;
        match (a, b) {
            (0xfff0, 0x1) => {
                match self.mem.get(x) {
                    Some(word) => cpu.registers[Register::Y] = *word,
                    None => unimplemented!(),
                }
                Ok(ONE_MS)
            }
            (0xfff0, 0x2) => {
                match self.mem.get_mut(x) {
                    Some(word) => *word &= cpu.registers[Register::Y],
                    None => unimplemented!(),
                }
                Ok(5 * ONE_MS)
            }
            (0xfff0, 0x3) => {
                self.mem = [0xffff; MEMORY_SIZE];
                Ok(10 * ONE_MS)
            }
            _ => self.inner.interrupt(cpu),
        }
    }

    fn tick(&mut self, cpu: &mut Cpu, current_tick: u64) -> Result<TickResult> {
        self.inner.tick(cpu, current_tick)
    }

    fn inspect(&self) {
        self.inner.inspect();
        println!("Eeprom: {:?}", self.mem);
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}
