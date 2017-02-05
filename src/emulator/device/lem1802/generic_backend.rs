use std::any::Any;
use std::fmt;
use std::sync::{Arc, Mutex};

use emulator::cpu;
use emulator::device::Result;
use emulator::device::lem1802;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum ScreenCommand {
    Show(Box<lem1802::RawScreen>),
    Hide,
}

pub struct ScreenBackend {
    // used for Drop
    #[allow(dead_code)]
    common: Arc<Mutex<Any + Send>>,
    send: Box<Fn(ScreenCommand) -> Result<()> + Send + 'static>,
}

impl ScreenBackend {
    pub fn new<T>(common: Arc<Mutex<Any + Send>>, sender: T) -> ScreenBackend
    where T: Fn(ScreenCommand) -> Result<()> + Send + 'static{
        ScreenBackend {
            common: common,
            send: Box::new(sender),
        }
    }
}

impl fmt::Debug for ScreenBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Glium backend")
    }
}

impl lem1802::Backend for ScreenBackend {
    fn tick<B: lem1802::Backend>(&self,
                                 cpu: &cpu::Cpu,
                                 lem: &lem1802::LEM1802<B>,
                                 tick_count: u64) -> Result<()> {
        // TODO: 10 fps for now by fear to fill the buffer
        if tick_count % 10_000 == 0 {
            self.try_show(cpu, lem)
        } else {
            Ok(())
        }
    }

    fn hide(&self) -> Result<()> {
        (self.send)(ScreenCommand::Hide)
    }

    fn show<B: lem1802::Backend>(&self,
                                 cpu: &cpu::Cpu,
                                 lem: &lem1802::LEM1802<B>) -> Result<()> {
        self.try_show(cpu, lem)
    }
}

impl ScreenBackend {
    fn try_show<B: lem1802::Backend>(&self,
                                     cpu: &cpu::Cpu,
                                     lem: &lem1802::LEM1802<B>) -> Result<()> {
        if let Some(screen) = lem.get_raw_screen(cpu) {
            (self.send)(ScreenCommand::Show(screen))
        } else {
            Ok(())
        }
    }
}
