use cpu::Cpu;

pub enum TickResult {
    Nothing,
    Interrupt
}

pub type InterruptDelay = u16;

pub trait Device {
    fn tick_rate(&self) -> u32;
    fn hardware_id(&self) -> u32;
    fn hardware_version(&self) -> u16;
    fn manufacturer(&self) -> u32;

    fn interrupt(&mut self, &mut Cpu) -> Result<InterruptDelay, ()>;
    fn tick(&mut self, &mut Cpu) -> TickResult;
}
