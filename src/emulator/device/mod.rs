pub mod clock;
pub mod dummy;
pub mod keyboard;
pub mod lem1802;
pub mod m35fd;

#[cfg(feature = "glium")]
pub mod glium_backend;

use std::any::Any;
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
            display("invalid hardware command: {:#x}", cmd)
        }
        BackendStopped(which: String) {
            description("the backend stopped")
            display("the {} backend stopped", which)
        }
    }
);

pub trait Device: Debug + Send + Any {
    fn hardware_id(&self) -> u32;
    fn hardware_version(&self) -> u16;
    fn manufacturer(&self) -> u32;

    fn interrupt(&mut self, &mut Cpu) -> Result<InterruptDelay>;
    fn tick(&mut self, &mut Cpu, current_tick: u64) -> Result<TickResult>;

    fn inspect(&self);
    fn as_any(&mut self) -> &mut Any;
}
