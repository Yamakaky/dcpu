use std::any::Any;

use enum_primitive::FromPrimitive;

use emulator::Cpu;
use emulator::device::*;
use types::Register;

const ONE_MS: u16 = 100;
const MEMORY_SIZE: usize = 16;
const DEFAULT_MEM_VALUE: [u16; MEMORY_SIZE] = [0xffff; MEMORY_SIZE];
const INT_VALUE: u16 = 0xfff0;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
enum Command {
    GET = 1,
    SET = 2,
    RESET = 3,
}
}

#[derive(Debug)]
pub struct Eeprom<D: Device> {
    mem: [u16; MEMORY_SIZE],
    inner: D,
}

impl<D: Device> Eeprom<D> {
    pub fn new(inner: D) -> Eeprom<D> {
        Eeprom {
            mem: DEFAULT_MEM_VALUE,
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
        if a == INT_VALUE {
            let b = cpu.registers[Register::B];
            let x = cpu.registers[Register::X] as usize;

            match Command::from_u16(b) {
                Some(Command::GET) => {
                    match self.mem.get(x) {
                        Some(word) => cpu.registers[Register::Y] = *word,
                        None => unimplemented!(),
                    }
                    Ok(ONE_MS)
                }
                Some(Command::SET) => {
                    match self.mem.get_mut(x) {
                        Some(word) => *word &= cpu.registers[Register::Y],
                        None => unimplemented!(),
                    }
                    Ok(5 * ONE_MS)
                }
                Some(Command::RESET) => {
                    self.mem = DEFAULT_MEM_VALUE;
                    Ok(10 * ONE_MS)
                }
                None => unimplemented!(),
            }
        } else {
            self.inner.interrupt(cpu)
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
