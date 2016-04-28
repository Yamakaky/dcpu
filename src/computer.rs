use cpu;
use device::*;

#[derive(Default)]
pub struct Computer {
    cpu: cpu::Cpu,
    devices: Vec<Box<Device>>,
    current_tick: u64,
}

impl Computer {
    pub fn new(cpu: cpu::Cpu) -> Computer {
        Computer {
            cpu: cpu,
            ..Default::default()
        }
    }

    pub fn add_device(&mut self, d: Box<Device>) {
        self.devices.push(d);
    }

    pub fn tick(&mut self) -> Result<(), cpu::Error> {
        try!(self.cpu.tick(&mut self.devices));

        for device in self.devices.iter_mut() {
            match device.tick(&mut self.cpu, self.current_tick) {
                TickResult::Nothing => (),
                TickResult::Interrupt(msg) =>
                    self.cpu.trigger_interrupt(msg),
            }
        }

        self.current_tick += 1;
        Ok(())
    }
}
