use emulator::cpu;
use emulator::device::{Device, TickResult};

#[derive(Default)]
pub struct Computer {
    pub cpu: cpu::Cpu,
    devices: Vec<Box<Device>>,
    pub current_tick: u64,
}

impl Computer {
    pub fn new(cpu: cpu::Cpu, devices: Vec<Box<Device>>) -> Computer {
        Computer {
            cpu: cpu,
            devices: devices,
            current_tick: 0,
        }
    }

    pub fn tick(&mut self) -> Result<(), cpu::Error> {
        try!(self.cpu.tick(&mut self.devices));

        for device in &mut self.devices {
            match try!(device.tick(&mut self.cpu, self.current_tick)) {
                TickResult::Nothing => (),
                TickResult::Interrupt(msg) => self.cpu.hardware_interrupt(msg),
            }
        }

        self.current_tick += 1;
        Ok(())
    }
}
