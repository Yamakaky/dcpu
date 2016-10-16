use dcpu::emulator::device::keyboard::mpsc_backend::KeyboardEvent;
use dcpu::emulator::device::lem1802::generic_backend::ScreenCommand;

/// Message sent by the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Lem1802(u16, ScreenCommand),
}

/// Message sent by a client
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    CreateCpu(Vec<DeviceType>),
    Keyboard(u16, KeyboardEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceType {
    Lem1802,
    Keyboard,
    Clock,
}
