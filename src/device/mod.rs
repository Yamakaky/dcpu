pub mod clock;
pub mod dummy;
pub mod keyboard;
pub mod lem1802;

use std::fmt::Debug;

use cpu::Cpu;

pub enum TickResult {
    Nothing,
    Interrupt(u16),
}

pub type InterruptDelay = u16;

pub trait Device: Debug {
    fn hardware_id(&self) -> u32;
    fn hardware_version(&self) -> u16;
    fn manufacturer(&self) -> u32;

    fn interrupt(&mut self, &mut Cpu) -> Result<InterruptDelay, ()>;
    fn tick(&mut self, &mut Cpu, current_tick: u64) -> TickResult;

    fn inspect(&self);
}
