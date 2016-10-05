use std::any::Any;
use std::fmt;
use std::rc::Rc;

use emulator::cpu;
use emulator::device::lem1802;

pub enum ScreenCommand {
    Show(Box<lem1802::Screen>),
    Hide,
}

pub struct ScreenBackend {
    // used for Drop
    #[allow(dead_code)]
    common: Rc<Any>,
    send: Box<Fn(ScreenCommand)>,
}

impl ScreenBackend {
    pub fn new<T: Fn(ScreenCommand) + 'static>(common: Rc<Any>,
                                               sender: T) -> ScreenBackend {
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
                                 tick_count: u64) {
        // TODO: 10 fps for now by fear to fill the buffer
        if tick_count % 10_000 == 0 {
            self.try_show(cpu, lem);
        }
    }

    fn hide(&self) {
        (self.send)(ScreenCommand::Hide)
    }

    fn show<B: lem1802::Backend>(&self,
                                 cpu: &cpu::Cpu,
                                 lem: &lem1802::LEM1802<B>) {
        self.try_show(cpu, lem);
    }
}

impl ScreenBackend {
    fn try_show<B: lem1802::Backend>(&self,
                                     cpu: &cpu::Cpu,
                                     lem: &lem1802::LEM1802<B>) {
        if let Some(screen) = lem.get_screen(cpu) {
            (self.send)(ScreenCommand::Show(screen))
        }
    }
}
