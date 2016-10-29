use std::{fmt, io};
use std::fs::File;
use std::path::Path;
use std::any::Any;

use enum_primitive::FromPrimitive;

use byteorder::{self, ReadBytesExt};
use emulator::Cpu;
use emulator::device::*;
use emulator::Ram;
use types::Register;

const NB_SECTORS_BY_TRACK: u16 = 18;
const NB_SECTORS_TOTAL: u16 = 1440;
const SECTOR_SIZE_WORD: u32 = 513;
const TRACK_SEEKING_TIME: u64 = 100_000 * 10_000 / 24;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
enum Command {
    POLL_DEVICE = 0,
    SET_INT = 1,
    READ_SECTOR = 2,
    WRITE_SECTOR = 3,
}
}

#[derive(Debug, Copy, Clone)]
enum StateCode {
    NoMedia = 0,
    Ready = 1,
    ReadyWP = 2,
    Busy = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ErrorCode {
    None = 0,
    Busy = 1,
    NoMedia = 2,
    Protected = 3,
    Eject = 4,
    #[allow(dead_code)]
    BadSector = 5,
    #[allow(dead_code)]
    Broken = 0xffff,
}

pub struct Floppy {
    data: Box<[Sector; NB_SECTORS_TOTAL as usize]>,
    write_protected: bool,
}

impl fmt::Debug for Floppy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("A floppy disk"))
    }
}

type Sector = [u16; SECTOR_SIZE_WORD as usize];

#[derive(Debug)]
pub struct M35fd {
    floppy: Option<Floppy>,
    last_error: ErrorCode,
    int_msg: u16,
    current_operation: Option<DiskOperation>,
    current_sector: u16,
    /// Indicate an int should be launched on next tick.
    do_int_next_tick: bool,
}

#[derive(Debug)]
struct DiskOperation {
    tick_delay: u64,
    sector: u16,
    address: u16,
    side: Side,
}

#[derive(Debug)]
enum Side {
    Read,
    Write,
}

impl Device for M35fd {
    fn hardware_id(&self) -> u32 {
        0x4fd524c5
    }

    fn hardware_version(&self) -> u16 {
        0x000b
    }

    fn manufacturer(&self) -> u32 {
        0x1eb37e91
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay> {
        let a = cpu.registers[Register::A];
        match try!(Command::from_u16(a)
                           .ok_or(ErrorKind::InvalidCommand(a))) {
            Command::POLL_DEVICE => {
                cpu.registers[Register::B] = if let Some(ref f) = self.floppy {
                    if self.current_operation.is_some() {
                        StateCode::Busy
                    } else if f.write_protected {
                        StateCode::ReadyWP
                    } else {
                        StateCode::Ready
                    }
                } else {
                    StateCode::NoMedia
                } as u16;
                cpu.registers[Register::C] = self.last_error as u16;
            }
            Command::SET_INT => self.int_msg = cpu.registers[Register::X],
            Command::READ_SECTOR => {
                self.start_user_command(cpu, |_floppy, sector, address, delay| {
                    let operation = DiskOperation {
                        tick_delay: delay,
                        sector: sector,
                        address: address,
                        side: Side::Read,
                    };
                    (ErrorCode::None, Some(operation))
                })
            }
            Command::WRITE_SECTOR => {
                self.start_user_command(cpu, |floppy, sector, address, delay| {
                    if floppy.write_protected {
                        (ErrorCode::Protected, None)
                    } else {
                        let operation = DiskOperation {
                            tick_delay: delay,
                            sector: sector,
                            address: address,
                            side: Side::Write,
                        };
                        (ErrorCode::None, Some(operation))
                    }
                })
            }
        }
        Ok(0)
    }

    fn tick(&mut self, cpu: &mut Cpu, _current_tick: u64) -> Result<TickResult> {
        let modification = if let Some(ref mut op) = self.current_operation {
            if let Some(ref mut f) = self.floppy {
                if op.tick_delay == 0 {
                    f.do_operation(op, &mut cpu.ram);
                    self.last_error = ErrorCode::None;
                    true
                } else {
                    op.tick_delay -= 1;
                    false
                }
            } else {
                self.last_error = ErrorCode::Eject;
                true
            }
        } else {
            false
        };

        if modification {
            self.current_operation = None;
        }

        Ok(if (modification | self.do_int_next_tick) && self.int_msg != 0 {
            self.do_int_next_tick = false;
            TickResult::Interrupt(self.int_msg)
        } else {
            TickResult::Nothing
        })
    }

    fn inspect(&self) {
        println!("m35fd");
        if self.int_msg != 0 {
            println!("Int message is 0x{:x}", self.int_msg);
            if let Some(ref floppy) = self.floppy {
                println!("Floppy loaded ({})", if floppy.write_protected {
                    "read-write"
                } else {
                    "read only"
                })
            } else {
                println!("No floppy loaded");
            }
            if self.current_operation.is_some() {
                println!("Disk operation in progress");
            } else {
                println!("No disk operation in progress");
            }
            println!("Last error: {:?}", self.last_error);
        } else {
            println!("Currently disabled")
        }
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}

impl Floppy {
    fn do_operation(&mut self, op: &DiskOperation, ram: &mut Ram) {
        match op.side {
            Side::Read => ram.copy(self.data[op.sector as usize].iter(),
                                   op.address),
            Side::Write => {
                for (from, to) in ram.iter_wrap(op.address)
                                     .zip(self.data[op.sector as usize]
                                          .iter_mut()) {
                    *to = *from;
                }
            }
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Floppy> {
        let mut input = try!(File::open(path));
        let words = input.iter_items::<u16, byteorder::LittleEndian>();
        let mut floppy = Floppy::default();
        for (from, to) in words.zip(floppy.data.iter_mut().flat_map(|s| s.iter_mut())) {
            *to = from;
        }
        Ok(floppy)
    }
}

impl Default for Floppy {
    fn default() -> Floppy {
        Floppy {
            data: Box::new([[0; SECTOR_SIZE_WORD as usize]; NB_SECTORS_TOTAL as usize]),
            write_protected: false,
        }
    }
}

impl M35fd {
    pub fn new<F: Into<Option<Floppy>>>(floppy: F) -> M35fd {
        M35fd {
            floppy: floppy.into(),
            last_error: ErrorCode::None,
            int_msg: 0,
            current_operation: None,
            current_sector: 0,
            do_int_next_tick: false,
        }
    }

    pub fn eject(&mut self) -> Option<Floppy> {
        self.pote();
        self.floppy.take()
    }

    pub fn load(&mut self, floppy: Floppy) {
        self.pote();
        self.floppy = Some(floppy);
    }

    fn pote(&mut self) {
        if self.current_operation.is_some() {
            self.current_operation = None;
            self.current_sector = 0;
            self.last_error = ErrorCode::Eject;
            self.do_int_next_tick = true;
        }
    }

    fn start_user_command<F>(&mut self, cpu: &mut Cpu, f: F)
    where F: Fn(&Floppy, u16, u16, u64) -> (ErrorCode, Option<DiskOperation>) {
        let sector = cpu.registers[Register::X];
        let address = cpu.registers[Register::Y];

        self.last_error = if sector >= NB_SECTORS_TOTAL {
            ErrorCode::BadSector
        } else if self.current_operation.is_some() {
            ErrorCode::Busy
        } else if let Some(ref floppy) = self.floppy {
            let delay = sector_distance(self.current_sector, sector);
            let (res, op) = f(floppy, sector, address, delay);
            self.current_operation = op;
            res
        } else {
            ErrorCode::NoMedia
        };

        cpu.registers[Register::B] =
            (self.last_error == ErrorCode::None) as u16;
    }
}

fn sector_distance(from: u16, to: u16) -> u64 {
    let sectors_to_skip = ((from % NB_SECTORS_BY_TRACK) as i16
                           - (to % NB_SECTORS_BY_TRACK) as i16).abs() as u64;
    let tracks_to_skip = ((from / NB_SECTORS_BY_TRACK) as i16
                          - (to / NB_SECTORS_BY_TRACK) as i16).abs() as u64;
    tracks_to_skip * TRACK_SEEKING_TIME
        + sectors_to_skip * TRACK_SEEKING_TIME / (NB_SECTORS_BY_TRACK as u64)
}
