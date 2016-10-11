use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, mpsc};

use emulator::device::keyboard;

#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum KeyboardEvent {
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
}

pub struct KeyboardBackend {
    // used for Drop
    #[allow(dead_code)]
    common: Arc<Mutex<Any + Send>>,
    receiver: mpsc::Receiver<KeyboardEvent>,
    key_pressed: [bool; 0x92],
}

impl KeyboardBackend {
    pub fn new(common: Arc<Mutex<Any + Send>>,
               receiver: mpsc::Receiver<KeyboardEvent>)
        -> KeyboardBackend {
        KeyboardBackend {
            common: common,
            receiver: receiver,
            key_pressed: [false; 0x92],
        }
    }
}

impl fmt::Debug for KeyboardBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Generic keyboard backend using mpsc")
    }
}

impl keyboard::Backend for KeyboardBackend {
    fn is_key_pressed(&mut self, key: keyboard::Key) -> bool {
        self.key_pressed[key.encode() as usize]
    }

    fn push_typed_keys(&mut self,
                       queue: &mut VecDeque<keyboard::Key>) -> bool {
        let mut new_keys = false;
        loop {
            match self.receiver.try_recv() {
                Ok(KeyboardEvent::KeyPressed(k)) => {
                    new_keys = true;
                    self.key_pressed[k.encode() as usize] = true;
                    queue.push_back(k);
                    if queue.len() > 8 {
                        queue.pop_front();
                    }
                }
                Ok(KeyboardEvent::KeyReleased(k)) => {
                    new_keys = true;
                    self.key_pressed[k.encode() as usize] = false;
                }
                Err(mpsc::TryRecvError::Empty) => return new_keys,
                Err(mpsc::TryRecvError::Disconnected) => panic!("Thread down"),
            }
        }
    }
}

