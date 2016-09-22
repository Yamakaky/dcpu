use std::collections::VecDeque;
use std::fmt::Debug;

use num::traits::FromPrimitive;

use cpu::Cpu;
use device::*;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    CLEAR_BUFFER = 0x0,
    GET_NEXT = 0x1,
    CHECK_KEY = 0x2,
    SET_INT = 0x3,
}
}

pub trait Backend: Debug {
    fn is_key_pressed(&mut self, key: Key) -> bool;
    fn push_typed_keys(&mut self, queue: &mut VecDeque<Key>) -> bool;
}

#[derive(Debug)]
pub struct Keyboard<B: Backend> {
    key_buffer: VecDeque<Key>,
    int_msg: u16,
    backend: B,
}

impl<B: Backend> Keyboard<B> {
    pub fn new(backend: B) -> Keyboard<B> {
        Keyboard {
            key_buffer: VecDeque::new(),
            int_msg: 0,
            backend: backend,
        }
    }
}

impl<B: Backend> Device for Keyboard<B> {
    fn hardware_id(&self) -> u32 {
        0x30cf7406
    }

    fn hardware_version(&self) -> u16 {
        1
    }

    fn manufacturer(&self) -> u32 {
        0x1c6c8b36
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay, ()> {
        let a = cpu.registers[0];
        let b = cpu.registers[1];
        match Command::from_u16(a) {
            Some(Command::CLEAR_BUFFER) => self.key_buffer.clear(),
            Some(Command::GET_NEXT) =>
                cpu.registers[2] = self.key_buffer
                                       .pop_front()
                                       .map(Key::encode)
                                       .unwrap_or(0),
            Some(Command::CHECK_KEY) => {
                let key = match Key::decode(b) {
                    Ok(k) => k,
                    Err(()) => return Err(()),
                };
                cpu.registers[2] = self.backend.is_key_pressed(key) as u16;
            },
            Some(Command::SET_INT) => self.int_msg = b,
            _ => return Err(()),
        }
        Ok(0)
    }

    fn tick(&mut self, _: &mut Cpu, _: u64) -> TickResult {
        if self.backend.push_typed_keys(&mut self.key_buffer) && self.int_msg != 0 {
            TickResult::Interrupt(self.int_msg)
        } else {
            TickResult::Nothing
        }
    }

    fn inspect(&self) {
        println!("Generic Keyboard");
        if self.int_msg == 0 {
            println!("Currently disabled");
        } else {
            println!("Int message is 0x{:x}", self.int_msg);
            println!("{} keys in the buffer", self.key_buffer.len());
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Key {
    Backspace,
    Return,
    Insert,
    Delete,
    ASCII(u16),
    Up,
    Down,
    Left,
    Right,
    Shift,
    Control,
}

impl Key {
    pub fn from_char(c: char) -> Result<Key, ()> {
        let n = c as u32 as u16;
        if 0x20 <= n && n <= 0x7f {
            Ok(Key::ASCII(n))
        } else {
            Err(())
        }
    }

    pub fn encode(self) -> u16 {
        match self {
            Key::Backspace => 0x10,
            Key::Return => 0x11,
            Key::Insert => 0x12,
            Key::Delete => 0x13,
            Key::ASCII(c) => c,
            Key::Up => 0x80,
            Key::Down => 0x81,
            Key::Left => 0x82,
            Key::Right => 0x83,
            Key::Shift => 0x90,
            Key::Control => 0x91,
        }
    }

    pub fn decode(c: u16) -> Result<Key, ()> {
        match c {
            0x10 => Ok(Key::Backspace),
            0x11 => Ok(Key::Return),
            0x12 => Ok(Key::Insert),
            0x13 => Ok(Key::Delete),
            0x80 => Ok(Key::Up),
            0x81 => Ok(Key::Down),
            0x82 => Ok(Key::Left),
            0x83 => Ok(Key::Right),
            0x90 => Ok(Key::Shift),
            0x91 => Ok(Key::Control),
            c if 0x20 <= c && c <= 0x7f => Ok(Key::ASCII(c)),
            _ => Err(())
        }
    }
}
