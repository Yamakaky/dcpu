use std::cmp::max;
use std::collections::VecDeque;
use std::default::Default;
use std::fmt;
use std::error::{self, Error as StdError};

use emulator::device::Device;
use types::*;
use types::Value::*;
use types::BasicOp::*;
use types::SpecialOp::*;

#[derive(Debug)]
pub enum Error {
    DecodeError(DecodeError),
    InvalidHardwareId(u16),
    InterruptError,
    InFire,
    Halted,
    Break(u16),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::DecodeError(ref e) => write!(f, "instruction decoding error: {}", e),
            Error::InvalidHardwareId(ref id) => write!(f, "invalid device id: {}", id),
            Error::Break(msg) => write!(f, "hardware breakpoint triggered with message {}", msg),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::DecodeError(ref e) => e.description(),
            Error::InvalidHardwareId(_) => "invalid hardware id",
            Error::Break(_) => "hardware breakpoint triggered",
            Error::InterruptError => "invalid hardware int",
            Error::InFire => "dcpu in fire, run for your lives!",
            Error::Halted => "cpu halted",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::DecodeError(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Error {
        Error::DecodeError(e)
    }
}

#[derive(Debug)]
pub enum CpuState {
    Executing,
    Waiting,
}

#[derive(Debug)]
pub enum OnDecodeError {
    Continue,
    Fail,
}

pub struct Cpu {
    pub ram: [u16; 0x10000],
    pub registers: [u16; 8],
    pub pc: u16,
    pub sp: u16,
    pub ex: u16,
    pub ia: u16,
    pub wait: u16,
    pub on_decode_error: OnDecodeError,
    pub check_if_cascade: bool,
    pub is_queue_enabled: bool,
    pub interrupts_queue: VecDeque<u16>,
    pub log_queue: VecDeque<u16>,
    pub halted: bool,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            ram: [0x03e0; 0x10000],
            registers: [0xdead; 8],
            pc: 0,
            sp: 0xffff,
            ex: 0,
            ia: 0,
            wait: 0,
            on_decode_error: OnDecodeError::Continue,
            check_if_cascade: true,
            is_queue_enabled: false,
            interrupts_queue: VecDeque::new(),
            log_queue: VecDeque::new(),
            halted: false,
        }
    }
}

impl Cpu {
    pub fn new(e: OnDecodeError) -> Cpu {
        let mut cpu = Cpu::default();
        cpu.on_decode_error = e;
        cpu
    }

    pub fn load(&mut self, data: &[u16], offset: u16) {
        for (i, d) in data.iter().enumerate() {
            self.ram[offset.wrapping_add(i as u16) as usize] = *d;
        }
    }

    pub fn load_ops(&mut self, ops: &[Instruction], mut offset: u16) {
        for op in ops {
            offset += op.encode(&mut self.ram[offset as usize..]);
        }
    }

    fn get(&mut self, i: Value) -> u16 {
        match i {
            Reg(r) => self.registers[r as usize],
            AtReg(r) => self.ram[(self.registers[r as usize]) as usize],
            AtRegPlus(r, off) => self.ram[off.wrapping_add(self.get(Reg(r))) as usize],
            Push => {
                let v = self.ram[self.sp as usize];
                self.sp = self.sp.wrapping_add(1);
                v
            },
            Peek => self.ram[self.sp as usize],
            Pick(n) => self.ram[self.sp.wrapping_add(n) as usize],
            SP => self.sp,
            PC => self.pc,
            EX => self.ex,
            AtAddr(off) => self.ram[off as usize],
            Litteral(n) => n
        }
    }

    fn set(&mut self, i: Value, val: u16) {
        match i {
            Reg(r) => self.registers[r as usize] = val,
            AtReg(r) => self.ram[(self.registers[r as usize]) as usize] = val,
            AtRegPlus(r, off) => self.ram[off.wrapping_add(self.get(Reg(r))) as usize] = val,
            Push => {
                self.sp = self.sp.wrapping_sub(1);
                self.ram[self.sp as usize] = val;
            },
            Peek => self.ram[self.sp as usize] = val,
            Pick(n) => self.ram[self.sp.wrapping_add(n) as usize] = val,
            SP => self.sp = val,
            PC => self.pc = val,
            EX => self.ex = val,
            AtAddr(off) => self.ram[off as usize] = val,
            Litteral(_) => ()
        }
    }

    pub fn tick(&mut self, devices: &mut [Box<Device>]) -> Result<CpuState, Error> {
        if self.halted {
            return Err(Error::Halted);
        }
        if self.wait != 0 {
            self.wait -= 1;
            trace!("Waiting");
            return Ok(CpuState::Waiting);
        }

        if !self.is_queue_enabled {
            if let Some(interrupt) = self.interrupts_queue.pop_front() {
                self.trigger_interrupt(interrupt);
            }
        }

        let pc = self.pc;
        let (words_used, instruction) = match self.decode(pc) {
            Ok(res) => res,
            Err(e) => match self.on_decode_error {
                OnDecodeError::Continue => {
                    warn!("Instruction decoding error: {:x}", self.ram[pc as usize]);
                    self.pc += 1;
                    return Ok(CpuState::Executing);
                },
                OnDecodeError::Fail => return Err(e.into()),
            }
        };
        self.pc = self.pc.wrapping_add(words_used);

        if self.check_if_cascade {
            trace!("Skipping cascade");
            self.check_if_cascade = instruction.is_if();
            if instruction.is_if() {
                return Ok(CpuState::Waiting);
            }
        }

        trace!("Executing {:?}", instruction);
        // BRK and HLT have a 0 delay
        self.wait = max(instruction.delay(), 1) - 1;
        try!(self.op(instruction, devices));

        Ok(CpuState::Executing)
    }

    fn decode(&mut self, offset: u16) -> Result<(u16, Instruction), DecodeError> {
        let bin = [
            self.get(AtAddr(offset)),
            self.get(AtAddr(offset.wrapping_add(1))),
            self.get(AtAddr(offset.wrapping_add(2)))
        ];
        Instruction::decode(&bin)
    }

    pub fn trigger_interrupt(&mut self, i: u16) {
        if self.ia != 0 {
            self.is_queue_enabled = true;
            let pc = self.get(PC);
            self.set(Push, pc);
            let a = self.get(Reg(Register::A));
            self.set(Push, a);
            let ia = self.ia;
            self.set(PC, ia);
            self.set(Reg(Register::A), i);
        }
    }

    fn op(&mut self, i: Instruction, devices: &mut [Box<Device>]) -> Result<(), Error> {
        match i {
            Instruction::BasicOp(op, b, a) => self.basic_op(op, b, a),
            Instruction::SpecialOp(op, a) => self.special_op(op, a, devices)
        }
    }

    fn basic_op(&mut self, op: BasicOp, b: Value, a: Value) -> Result<(), Error> {
        match op {
            SET => self.op_set(b, a),
            ADD => self.op_add(b, a),
            SUB => self.op_sub(b, a),
            MUL => self.op_mul(b, a),
            MLI => self.op_mli(b, a),
            DIV => self.op_div(b, a),
            DVI => self.op_dvi(b, a),
            MOD => self.op_mod(b, a),
            MDI => self.op_mdi(b, a),
            AND => self.op_and(b, a),
            BOR => self.op_bor(b, a),
            XOR => self.op_xor(b, a),
            SHR => self.op_shr(b, a),
            ASR => self.op_asr(b, a),
            SHL => self.op_shl(b, a),
            IFB => self.op_ifb(b, a),
            IFC => self.op_ifc(b, a),
            IFE => self.op_ife(b, a),
            IFN => self.op_ifn(b, a),
            IFG => self.op_ifg(b, a),
            IFA => self.op_ifa(b, a),
            IFL => self.op_ifl(b, a),
            IFU => self.op_ifu(b, a),
            ADX => self.op_adx(b, a),
            SBX => self.op_sbx(b, a),
            STI => self.op_sti(b, a),
            STD => self.op_std(b, a)
        }
    }

    fn special_op(&mut self, op: SpecialOp, a: Value, devices: &mut [Box<Device>]) -> Result<(), Error> {
        match op {
            JSR => self.op_jsr(a),
            INT => self.op_int(a),
            IAG => self.op_iag(a),
            IAS => self.op_ias(a),
            RFI => self.op_rfi(a),
            IAQ => self.op_iaq(a),
            HWN => self.op_hwn(a, devices),
            HWQ => self.op_hwq(a, devices),
            HWI => self.op_hwi(a, devices),
            LOG => self.op_log(a),
            BRK => self.op_brk(a),
            HLT => self.op_hlt(),
        }
    }

    fn op_set(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.set(b, val_a);
        Ok(())
    }

    fn op_add(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow) = val_b.overflowing_add(val_a);
        self.set(b, new_b);
        self.ex = overflow as u16;
        Ok(())
    }

    fn op_sub(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow) = val_b.overflowing_sub(val_a);
        self.set(b, new_b);
        self.ex = if overflow {0xffff} else {0};
        Ok(())
    }

    fn op_mul(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as u32;
        let val_b = self.get(b) as u32;
        let new_b = val_a * val_b;
        self.set(b, new_b as u16);
        self.ex = (new_b >> 16) as u16;
        Ok(())
    }

    fn op_mli(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as i16 as i32;
        let val_b = self.get(b) as i16 as i32;
        let new_b = (val_a * val_b) as u32;
        self.set(b, new_b as u16);
        self.ex = (new_b >> 16) as u16;
        Ok(())
    }

    fn op_div(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        if val_a == 0 {
            self.set(b, 0);
            self.ex = 0;
        } else {
            let val_b = self.get(b);
            self.set(b, val_b / val_a);
            self.ex = ((val_b as u32) << 16 / val_a) as u16;
        }
        Ok(())
    }

    fn op_dvi(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as i16;
        if val_a == 0 {
            self.set(b, 0);
            self.ex = 0;
        } else {
            let val_b = self.get(b) as i16;
            self.set(b, (val_b / val_a) as u16);
            self.ex = ((val_b as i32) << 16 / val_a) as u16;
        }
        Ok(())
    }

    fn op_mod(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        if val_a == 0 {
            self.set(b, 0);
        } else {
            let val_b = self.get(b);
            self.set(b, val_b % val_a);
        }
        Ok(())
    }

    fn op_mdi(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as i16;
        if val_a == 0 {
            self.set(b, 0);
        } else {
            let val_b = self.get(b) as i16;
            self.set(b, (val_b % val_a) as u16);
        }
        Ok(())
    }

    fn op_and(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b & val_a);
        Ok(())
    }

    fn op_bor(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b | val_a);
        Ok(())
    }

    fn op_xor(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b ^ val_a);
        Ok(())
    }

    fn op_shr(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b >> val_a);
        self.ex = (((val_b as u32) << 16) >> val_a) as u16;
        Ok(())
    }

    fn op_asr(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b) as i16;
        self.set(b, (val_b >> val_a) as u16);
        self.ex = (((val_b as i32) << 16) >> val_a) as u16;
        Ok(())
    }

    fn op_shl(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b << val_a);
        self.ex = (((val_b as u32) << val_a) >> 16) as u16;
        Ok(())
    }

    fn exec_if(&mut self, cond: bool) -> Result<(), Error> {
        if !cond {
            let next_i = self.pc;
            let (offset, _) = try!(self.decode(next_i));
            self.pc = self.pc.wrapping_add(offset);
            self.check_if_cascade = true;
            self.wait += 1;
        }
        Ok(())
    }

    fn op_ifb(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if((val_a & val_b) != 0)
    }

    fn op_ifc(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if((val_a & val_b) == 0)
    }

    fn op_ife(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_a == val_b)
    }

    fn op_ifn(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_a != val_b)
    }

    fn op_ifg(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_a > val_b)
    }

    fn op_ifa(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as i16;
        let val_b = self.get(b) as i16;
        self.exec_if(val_a > val_b)
    }

    fn op_ifl(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_a < val_b)
    }

    fn op_ifu(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a) as i16;
        let val_b = self.get(b) as i16;
        self.exec_if(val_a < val_b)
    }

    fn op_adx(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let ex = self.ex;
        self.ex = 0;
        let (new_b, overflow) = val_b.overflowing_add(val_a);
        if overflow {
            self.ex = 1;
        }
        let (new_b, overflow) = new_b.overflowing_add(ex);
        if overflow {
            self.ex = 1;
        }
        self.set(b, new_b);
        Ok(())
    }

    fn op_sbx(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let ex = self.ex;
        self.ex = 0;
        let (new_b, overflow) = val_b.overflowing_sub(val_a);
        if overflow {
            self.ex = 0xffff;
        }
        let (new_b, overflow) = new_b.overflowing_add(ex);
        if overflow {
            self.ex = 0xffff;
        }
        self.set(b, new_b);
        Ok(())
    }

    fn op_sti(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.set(b, val_a);
        self.registers[Register::I as usize] =
            self.registers[Register::I as usize].wrapping_add(1);
        self.registers[Register::J as usize] =
            self.registers[Register::J as usize].wrapping_add(1);
        Ok(())
    }

    fn op_std(&mut self, b: Value, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.set(b, val_a);
        self.registers[Register::I as usize] =
            self.registers[Register::I as usize].wrapping_sub(1);
        self.registers[Register::J as usize] =
            self.registers[Register::J as usize].wrapping_sub(1);
        Ok(())
    }

    fn op_jsr(&mut self, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        let new_pc = Litteral(self.pc);
        try!(self.op_set(Push, new_pc));
        self.pc = val_a;
        Ok(())
    }

    fn op_int(&mut self, a: Value) -> Result<(), Error> {
        if self.ia != 0 {
            if self.interrupts_queue.len() >= 256 {
                return Err(Error::InFire);
            }
            let val_a = self.get(a);
            self.interrupts_queue.push_back(val_a);
        }
        Ok(())
    }

    fn op_iag(&mut self, a: Value) -> Result<(), Error> {
        let ia = self.ia;
        self.set(a, ia);
        Ok(())
    }

    fn op_ias(&mut self, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.ia = val_a;
        Ok(())
    }

    fn op_rfi(&mut self, _: Value) -> Result<(), Error> {
        self.is_queue_enabled = false;
        let v1 = self.get(Push);
        self.set(Reg(Register::A), v1);
        let v2 = self.get(Push);
        self.set(PC, v2);
        Ok(())
    }

    fn op_iaq(&mut self, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.is_queue_enabled = val_a == 0;
        Ok(())
    }

    fn op_hwn(&mut self, a: Value, devices: &mut [Box<Device>]) -> Result<(), Error> {
        let nb_devices = devices.len();
        self.set(a, nb_devices as u16);
        Ok(())
    }

    fn op_hwq(&mut self, a: Value, devices: &mut [Box<Device>]) -> Result<(), Error> {
        let val_a = self.get(a) as usize;

        if val_a < devices.len() {
            let id = devices[val_a].hardware_id();
            let version = devices[val_a].hardware_version();
            let manufacturer = devices[val_a].manufacturer();

            self.set(Reg(Register::A), id as u16);
            self.set(Reg(Register::B), (id >> 16) as u16);
            self.set(Reg(Register::C), version);
            self.set(Reg(Register::X), manufacturer as u16);
            self.set(Reg(Register::Y), (manufacturer >> 16) as u16);
            Ok(())
        } else {
            Err(Error::InvalidHardwareId(val_a as u16))
        }
    }

    fn op_hwi(&mut self, a: Value, devices: &mut [Box<Device>]) -> Result<(), Error> {
        let val_a = self.get(a) as usize;

        if val_a < devices.len() {
            self.wait += try!(devices[val_a].interrupt(self).map_err(|_| Error::InterruptError));
            Ok(())
        } else {
            Err(Error::InvalidHardwareId(val_a as u16))
        }
    }

    fn op_log(&mut self, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        self.log_queue.push_back(val_a);
        Ok(())
    }

    fn op_brk(&mut self, a: Value) -> Result<(), Error> {
        let val_a = self.get(a);
        Err(Error::Break(val_a))
    }

    fn op_hlt(&mut self) -> Result<(), Error> {
        self.halted = true;
        Err(Error::Halted)
    }
}
