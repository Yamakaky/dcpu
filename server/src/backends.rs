use std::sync::{mpsc, Arc, Mutex};

use tokio;

use dcpu::emulator::{Computer, Cpu};
use dcpu::emulator::device::{keyboard, lem1802, clock, Device};
use dcpu::emulator::device::keyboard::mpsc_backend::KeyboardBackend;
use dcpu::emulator::device::lem1802::generic_backend::ScreenBackend;

use msg::{self, ClientMessage, ServerMessage};

pub fn new_computer(devices_types: Vec<msg::DeviceType>,
                    tx: tokio::channel::Sender<msg::ServerMessage>)
    -> (Computer, BackendControler) {
    let mut devices = vec![];
    let mut controlers = vec![];

    let common = Arc::new(Mutex::new(()));
    for dev_type in &devices_types {
        let (dev, controler): (Box<Device>, _) = match *dev_type {
            msg::DeviceType::Lem1802 => {
                let tx = tx.clone();
                let callback =
                    move |s| tx.send(msg::ServerMessage::Lem1802(0, s))
                               .unwrap();
                let screen_backend = ScreenBackend::new(common.clone(),
                                                        callback);
                (Box::new(lem1802::LEM1802::new(screen_backend)), None)
            }
            msg::DeviceType::Keyboard => {
                let (tx2, rx2) = mpsc::channel();
                let keyboard_backend = KeyboardBackend::new(common.clone(),
                                                            rx2);
                (Box::new(keyboard::Keyboard::new(keyboard_backend)),
                 Some(Item::Keyboard(tx2)))
            }
            msg::DeviceType::Clock =>
                (Box::new(clock::Clock::new(100_000)), None),
        };
        devices.push(dev);
        controlers.push(controler);
    }

    (Computer::new(Cpu::default(), devices), BackendControler::new(controlers))
}

pub struct BackendControler {
    devices: Box<[Option<Item>]>,
}

impl BackendControler {
    fn new(devices: Vec<Option<Item>>) -> BackendControler {
        BackendControler {
            devices: devices.into_boxed_slice(),
        }
    }
    pub fn dispatch_server(&self, msg: ClientMessage) {
        println!("Dispatching {:?}", msg);
        match msg {
            ClientMessage::Keyboard(id, event) => {
                match self.devices.get(id as usize) {
                    Some(&Some(Item::Keyboard(ref sender))) =>
                        sender.send(event).unwrap(),
                    _ => println!("Invalid device id: {:?}", msg),
                }
            }
            ClientMessage::CreateCpu(_) => panic!("Too late to create a cpu!"),
        }
    }

    pub fn dispatch_client(&self, msg: ServerMessage) {
        match msg {
            ServerMessage::Lem1802(id, screen) => {
                match *self.devices[id as usize].as_ref().unwrap() {
                    Item::Lem1802(ref sender) => sender.send(screen).unwrap(),
                    _ => unreachable!(),
                }
            }
        }
    }
}

enum Item {
    Lem1802(mpsc::Sender<lem1802::generic_backend::ScreenCommand>),
    Keyboard(mpsc::Sender<keyboard::mpsc_backend::KeyboardEvent>),
}
