use std::any::Any;
use std::sync::mpsc;
use std::result::Result as StdResult;

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

#[allow(dead_code)]
enum TransmitError {
    Success = 0x0,
    // TODO
    PortBusy = 0x1,
    Overflow = 0x2,
    Unconnected = 0x3,
    // TODO
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
struct Message {
    port: usize,
    data: u16,
}

#[derive(Debug, Copy, Clone)]
enum Buffer<I> {
    Zero,
    One(I),
    Two(I, I),
}

impl<I: Copy> Buffer<I> {
    fn push(&mut self, val: I) -> bool {
        use self::Buffer::*;
        match *self {
            Zero => {
                *self = One(val);
                true
            }
            One(a) => {
                *self = Two(val, a);
                true
            }
            Two(a, _) => {
                *self = Two(val, a);
                false
            },
        }
    }

    fn pop(&mut self) -> Option<I> {
        use self::Buffer::*;
        match *self {
            Zero => None,
            One(a) => {
                *self = Zero;
                Some(a)
            }
            Two(a, b) => {
                *self = One(a);
                Some(b)
            }
        }
    }

    fn size(&self) -> usize {
        use self::Buffer::*;
        match *self {
            Zero => 0,
            One(..) => 1,
            Two(..) => 2,
        }
    }
}

impl<I> Default for Buffer<I> {
    fn default() -> Buffer<I> {
        Buffer::Zero
    }
}

#[derive(Debug)]
pub struct HIC {
    ports: Box<[Port]>,
    recv: mpsc::Receiver<Message>,
    // Used only for `connect()`
    send: mpsc::Sender<Message>,
    int_msg_recv: u16,
    int_msg_transmit: u16,
    send_buffer: Buffer<(usize, u16)>,
}

impl HIC {
    pub fn new(nb_ports: usize) -> Option<HIC> {
        if nb_ports == 8 || nb_ports == 16 || nb_ports == 32 {
            let mut ports = Vec::with_capacity(nb_ports);
            for _ in 0..nb_ports {
                ports.push(Port::default());
            }
            let (tx, rx) = mpsc::channel();
            Some(HIC {
                ports: ports.into_boxed_slice(),
                recv: rx,
                send: tx,
                int_msg_recv: 0,
                int_msg_transmit: 0,
                send_buffer: Buffer::default(),
            })
        } else {
            None
        }
    }

    pub fn connect(&mut self, other: &mut HIC) -> StdResult<(), ()> {
        let my_ports_free =
            self.ports.iter_mut().enumerate().filter(|&(_, ref p)| !p.is_connected()).next();
        let it_ports_free =
            other.ports.iter_mut().enumerate().filter(|&(_, ref p)| !p.is_connected()).next();
        if let (Some((my_id, my_free)), Some((other_id, other_free))) = (my_ports_free, it_ports_free) {
            my_free.connection = Some((other_id, other.send.clone()));
            other_free.connection = Some((my_id, self.send.clone()));
            Ok(())
        } else {
            Err(())
        }
    }

    fn is_sending(&self) -> bool {
        // TODO
        self.send_buffer.size() != 0
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
                    ((self.send_buffer.size() == 0) as u16) << 4 |
                    ((self.is_sending()) as u16) << 5 |
                    ((first_with_data != 0xffff) as u16) << 6 |
                    ((self.int_msg_recv != 0) as u16) << 14 |
                    ((self.int_msg_transmit != 0) as u16) << 15;
                cpu.registers[Register::A] = port_status | general_status;
                cpu.registers[Register::C] = first_with_data as u16;
            }
            Some(Command::RECEIVE) => {
                let (res, err) =
                    if let Some(port) = self.ports.get_mut(port_number) {
                        if let Some(val) = port.recv_buffer.pop() {
                            let err = if port.overflowed {
                                port.overflowed = false;
                                ReceiveError::Overflow
                            } else {
                                ReceiveError::Success
                            };
                            (val, err)
                        } else {
                            (0, ReceiveError::NoData)
                        }
                    } else {
                        // Should be WrongPort
                        (0, ReceiveError::Fail)
                    };
                cpu.registers[Register::B] = res;
                cpu.registers[Register::C] = err as u16;
            }
            Some(Command::TRANSMIT) => {
                let val = cpu.registers[Register::B];
                let connected = self.ports
                                    .get_mut(port_number)
                                    .map(|port| port.is_connected())
                                    .unwrap_or(false);
                cpu.registers[Register::C] = if connected {
                    if self.send_buffer.push((port_number, val)) {
                        TransmitError::Success
                    } else {
                        TransmitError::Overflow
                    }
                } else {
                    TransmitError::Unconnected
                } as u16;
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
        if let Some((port_num, val)) = self.send_buffer.pop() {
            try!(self.ports[port_num].try_send(val));
            if self.int_msg_transmit != 0 {
                return Ok(TickResult::Interrupt(self.int_msg_transmit));
            }
        }

        loop {
            match self.recv.try_recv() {
                Ok(Message { port, data }) => {
                    self.ports[port].recv(data);
                    if self.int_msg_recv != 0 {
                        return Ok(TickResult::Interrupt(self.int_msg_recv));
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(e) => try!(Err(ErrorKind::BackendStopped(format!("{}", e)))),
            }
        }

        Ok(TickResult::Nothing)
    }

    fn inspect(&self) {
        println!("{:?}", self);
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}

#[derive(Debug, Default)]
pub struct Port {
    connection: Option<(usize, mpsc::Sender<Message>)>,
    name: [u16; 8],
    recv_buffer: Buffer<u16>,
    overflowed: bool,
}

impl Port {
    fn try_send(&mut self, val: u16) -> Result<()> {
        if let Some((port_num, ref sender)) = self.connection {
            match sender.send(Message {
                port: port_num,
                data: val,
            }) {
                Ok(()) => Ok(()),
                Err(_) => try!(Err(ErrorKind::BackendStopped("hic".into()))),
            }
        } else {
            unreachable!()
        }
    }

    fn recv(&mut self, data: u16) {
        self.overflowed = !self.recv_buffer.push(data);
    }

    fn status(&self) -> u16 {
        self.is_busy() as u16 |
        (self.is_connected() as u16) << 1 |
        (self.has_data() as u16) << 2
    }

    fn has_data(&self) -> bool {
        self.recv_buffer.size() != 0
    }

    fn is_busy(&self) -> bool {
        // TODO
        false
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}
