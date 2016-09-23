pub mod clock;
pub mod dummy;
pub mod keyboard;
pub mod lem1802;

pub mod glium_backend;

use std::fmt::Debug;

use emulator::cpu::Cpu;

pub enum TickResult {
    Nothing,
    Interrupt(u16),
}

pub type InterruptDelay = u16;

error_chain!(
    errors {
        InvalidCommand(cmd: u16) {
            description("invalid hardware command")
            display("invalid hardware command: {}", cmd)
        }
    }
);

pub trait Device: Debug {
    fn hardware_id(&self) -> u32;
    fn hardware_version(&self) -> u16;
    fn manufacturer(&self) -> u32;

    fn interrupt(&mut self, &mut Cpu) -> Result<InterruptDelay>;
    fn tick(&mut self, &mut Cpu, current_tick: u64) -> TickResult;

    fn inspect(&self);
}
