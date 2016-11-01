// tmp while things are unimplemented
#![allow(dead_code)]

use std::any::Any;

use enum_primitive::FromPrimitive;

use emulator::Cpu;
use emulator::device::*;
use types::Register;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    QUERY_STATUS = 0x0,
    RECEIVE = 0x1,
    TRANSMIT = 0x2,
    CONFIGURE = 0x3,
    LOAD_PORT_NAME = 0x4,
}
}

enum ReceiveError {
    Success = 0x0,
    Overflow = 0x1,
    Fail = 0x2,
    NoData = 0x3,
}

enum TransmitError {
    Success = 0x0,
    PortBusy = 0x1,
    Overflow = 0x2,
    Unconnected = 0x3,
    HICBusy = 0x4,
}

enum LoadNameError {
    Success = 0x0,
    OutOfBound = 0x1,
    InvalidAddress = 0x2,
}

#[derive(Debug)]
pub enum NumberPorts {
    N8,
    N16,
    N32,
}

#[derive(Debug)]
pub struct HIC {
    ports: Box<[Port]>,
    int_msg_recv: u16,
    int_msg_transmit: u16,
    send_buffer: [u16; 2],
    send_buffer_size: usize,
}

impl HIC {
    pub fn new(number_ports: NumberPorts) -> HIC {
        HIC {
            ports: match number_ports {
                NumberPorts::N8 => vec![Port::default(); 8],
                NumberPorts::N16 => unimplemented!(),
                NumberPorts::N32 => unimplemented!(),
            }.into_boxed_slice(),
            int_msg_recv: 0,
            int_msg_transmit: 0,
            send_buffer: [0; 2],
            send_buffer_size: 0,
        }
    }

    fn is_sending(&self) -> bool {
        unimplemented!()
    }
}

impl Device for HIC {
    fn hardware_id(&self) -> u32 {
        0xe0239088
    }

    fn hardware_version(&self) -> u16 {
        match self.ports.len() {
            8 => 0x0442,
            16 => 0x0444,
            32 => 0x0448,
            _ => unreachable!(),
        }
    }

    fn manufacturer(&self) -> u32 {
        0xa87c900e
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay> {
        let port_number = cpu.registers[Register::C] as usize;
        match Command::from_u16(cpu.registers[Register::A]) {
            Some(Command::QUERY_STATUS) => {
                let port_status = self.ports
                                      .get(port_number)
                                      .map(Port::status)
                                      .unwrap_or(1 << 3);
                let first_with_data = self.ports
                                          .iter()
                                          .position(Port::has_data)
                                          .unwrap_or(0xffff);
                let general_status =
                    ((self.send_buffer_size == 0) as u16) << 4 |
                    ((self.is_sending()) as u16) << 5 |
                    ((first_with_data != 0xffff) as u16) << 6 |
                    ((self.int_msg_recv != 0) as u16) << 14 |
                    ((self.int_msg_transmit != 0) as u16) << 15;
                cpu.registers[Register::A] = port_status | general_status;
                cpu.registers[Register::C] = first_with_data as u16;
            }
            Some(Command::RECEIVE) => {
                let (res, err) = self.ports
                                     .get_mut(port_number)
                                     .map(|port| port.recv())
                                     .unwrap_or((0, ReceiveError::Fail));
                cpu.registers[Register::B] = res;
                cpu.registers[Register::C] = err as u16;
            }
            Some(Command::TRANSMIT) => {
                let val = cpu.registers[Register::B];
                cpu.registers[Register::C] =
                    self.ports
                        .get_mut(port_number)
                        .map(|port| port.send(val))
                        // should be OOB
                        .unwrap_or_else(|| unreachable!()) as u16;
            }
            Some(Command::CONFIGURE) => {
                self.int_msg_recv = cpu.registers[Register::B];
                self.int_msg_transmit = cpu.registers[Register::C];
                cpu.registers[Register::C] = 0;
            }
            Some(Command::LOAD_PORT_NAME) => {
                let err = match self.ports.get(port_number) {
                    Some(port) => {
                        let dst_addr = cpu.registers[Register::B];
                        if dst_addr <= 0xffff - 8 {
                            cpu.ram.copy(port.name.iter(), dst_addr);
                            LoadNameError::Success
                        } else {
                            LoadNameError::InvalidAddress
                        }
                    }
                    None => LoadNameError::OutOfBound,
                };
                cpu.registers[Register::C] = err as u16;
            }
            None => unimplemented!(),
        }
        Ok(0)
    }

    fn tick(&mut self, _cpu: &mut Cpu, _current_tick: u64) -> Result<TickResult> {
        unimplemented!()
    }

    fn inspect(&self) {
        unimplemented!()
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}

#[derive(Debug, Default, Clone)]
struct Port {
    name: [u16; 8],
    recv_buffer: [u16; 2],
    recv_buffer_size: usize,
}

impl Port {
    fn send(&mut self, _data: u16) -> TransmitError {
        unimplemented!()
    }

    fn recv(&mut self) -> (u16, ReceiveError) {
        unimplemented!()
    }

    fn status(&self) -> u16 {
        self.is_busy() as u16 |
        (self.is_connected() as u16) << 1 |
        ((self.recv_buffer_size != 0) as u16) << 2
    }

    fn has_data(&self) -> bool {
        unimplemented!()
    }

    fn is_busy(&self) -> bool {
        unimplemented!()
    }

    fn is_connected(&self) -> bool {
        unimplemented!()
    }
}
