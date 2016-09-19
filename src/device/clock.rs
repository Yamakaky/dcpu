use num::traits::FromPrimitive;

use cpu::Cpu;
use device::*;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    SET_SPEED = 0x0,
    GET_TICKS = 0x1,
    SET_INT = 0x2
}
}

#[derive(Debug)]
pub struct Clock {
    /// Should be 100_000
    ticks_per_second: u64,
    speed: u16,
    int_msg: u16,
    last_call: u64,
}

impl Clock {
    pub fn new(ticks_per_second: u64) -> Clock {
        Clock {
            ticks_per_second: ticks_per_second,
            speed: 0,
            int_msg: 0,
            last_call: 0,
        }
    }
}

impl Device for Clock {
    fn hardware_id(&self) -> u32 {
        0x12d0b402
    }

    fn hardware_version(&self) -> u16 {
        1
    }

    fn manufacturer(&self) -> u32 {
        0x1c6c8b36
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay, ()> {
        let a = cpu.registers[0];
        let b = cpu.registers[1];
        match Command::from_u16(a) {
            Some(Command::SET_SPEED) => self.speed = b,
            Some(Command::GET_TICKS) => {
                cpu.registers[2] = self.last_call as u16;
                self.last_call = 0;
            },
            Some(Command::SET_INT) => self.int_msg = b,
            None => return Err(())
        }

        Ok(0)
    }

    fn tick(&mut self, _: &mut Cpu, current_tick: u64) -> TickResult {
        if self.speed != 0 && self.int_msg != 0 {
            if current_tick % (60 * self.ticks_per_second / self.speed as u64) == 0 {
                self.last_call += 1;
                return TickResult::Interrupt(self.int_msg);
            }
        }

        return TickResult::Nothing;
    }
}
