use std::any::Any;

use enum_primitive::FromPrimitive;
use time::{empty_tm, now, Duration, Tm};

use emulator::cpu::Cpu;
use emulator::Registers;
use emulator::device::*;
use types::Register;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    SET_SPEED = 0x0,
    GET_TICKS = 0x1,
    SET_INT = 0x2,
    REAL_TIME = 0x10,
    RUN_TIME = 0x11,
    SET_REAL_TIME = 0x12,
    RESET = 0xffff,
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
    /// Difference between real-life and in-game time
    delta_time: Duration,
}

impl Clock {
    pub fn new(ticks_per_second: u64) -> Clock {
        Clock {
            ticks_per_second: ticks_per_second,
            speed: 0,
            int_msg: 0,
            last_call: 0,
            next_tick: 0,
            delta_time: Duration::zero(),
        }
    }
}

impl Device for Clock {
    #[cfg(feature = "old-device-id")]
    fn hardware_id(&self) -> u32 {
        0x12d0b402
    }

    #[cfg(not(feature = "old-device-id"))]
    fn hardware_id(&self) -> u32 {
        0x12d1b402
    }
    fn hardware_version(&self) -> u16 {
        2
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
            Command::REAL_TIME =>
                encode_time(&mut cpu.registers, now() + self.delta_time),
            Command::RUN_TIME =>
                encode_time(&mut cpu.registers, empty_tm() + self.delta_time),
            Command::SET_REAL_TIME =>
                self.delta_time = now() - decode_time(&cpu.registers),
            Command::RESET => *self = Clock::new(self.ticks_per_second),
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

fn encode_time(regs: &mut Registers, time: Tm) {
    regs[Register::B] = time.tm_year as u16;
    regs[Register::C] =
        (time.tm_mon as u16 + 1) << 8 | time.tm_mday as u16;
    regs[Register::X] =
        (time.tm_hour as u16) << 8 | time.tm_min as u16;
    regs[Register::Y] = time.tm_sec as u16;
    regs[Register::Z] = (time.tm_nsec / 1_000_000) as u16;
}

fn decode_time(regs: &Registers) -> Tm {
    Tm {
        tm_year: regs[Register::B] as i32,
        tm_mon: ((regs[Register::C] >> 8) - 1) as i32,
        tm_mday: (regs[Register::C] & 0xff) as i32,
        tm_hour: (regs[Register::X] >> 8) as i32,
        tm_min: (regs[Register::X] & 0xff) as i32,
        tm_sec: regs[Register::Y] as i32,
        tm_nsec: (regs[Register::Z] as i32) * 1_000_000,
        ..empty_tm()
    }
}
