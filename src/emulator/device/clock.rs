use std::any::Any;

use enum_primitive::FromPrimitive;

use emulator::cpu::Cpu;
use emulator::device::*;
use types::Register;

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
    /// Which future CPU tick should we tick on (optimisation)
    next_tick: u64,
}

impl Clock {
    pub fn new(ticks_per_second: u64) -> Clock {
        Clock {
            ticks_per_second: ticks_per_second,
            speed: 0,
            int_msg: 0,
            last_call: 0,
            next_tick: 0,
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

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay> {
        let a = cpu.registers[Register::A];
        let b = cpu.registers[Register::B];
        match try!(Command::from_u16(a)
                           .ok_or(ErrorKind::InvalidCommand(a))) {
            Command::SET_SPEED => self.speed = b,
            Command::GET_TICKS => {
                cpu.registers[Register::C] = self.last_call as u16;
                self.last_call = 0;
            },
            Command::SET_INT => self.int_msg = b,
        }

        Ok(0)
    }

    fn tick(&mut self, _: &mut Cpu, current_tick: u64) -> Result<TickResult> {
        Ok(if self.speed != 0 && self.int_msg != 0 &&
              current_tick >= self.next_tick {
            self.last_call += 1;
            // If we calculate the expression between parens in the `if`
            // condition, we loose 15% perfs.
            self.next_tick = current_tick +
                ((self.speed as u64) * self.ticks_per_second / 60);
            TickResult::Interrupt(self.int_msg)
        } else {
            TickResult::Nothing
        })
    }

    fn inspect(&self) {
        println!("Generic clock");
        if self.speed == 0 || self.int_msg == 0 {
            println!("Currently disabled");
        } else {
            println!("FPS: {}", 60. / (self.speed as f32));
            println!("Int message is 0x{:x}", self.int_msg);
            println!("Last call was {} ticks ago", self.last_call);
        }
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}
