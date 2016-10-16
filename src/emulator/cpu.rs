use std::cmp::max;
use std::collections::VecDeque;
use std::default::Default;
use std::num::Wrapping;

use emulator::device::{self, Device};
use emulator::ram::Ram;
use emulator::registers::Registers;
use types::*;
use types::Value::*;
use types::BasicOp::*;
use types::SpecialOp::*;

error_chain!(
    links {
        device::Error, device::ErrorKind, InterruptError;
    }
    foreign_links {
        DecodeError, DecodeError;
    }
    errors {
        InvalidHardwareId(id: u16) {
            display("invalid device id: {:#x}", id)
            description("invalid device id")
        }
        InFire {
            display("dcpu in fire")
            description("dcpu in fire")
        }
        Halted {
            display("cpu halted")
            description("cpu halted")
        }
        Break(msg: u16) {
            display("hardware breakpoint triggered with message {:#x}", msg)
            description("hardware breakpoint triggered")
        }
    }
);

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
    pub ram: Ram,
    pub registers: Registers,
    pub pc: Wrapping<u16>,
    pub sp: Wrapping<u16>,
    pub ex: u16,
    pub ia: u16,
    pub wait: u16,
    pub on_decode_error: OnDecodeError,
    pub is_queue_enabled: bool,
    pub interrupts_queue: VecDeque<u16>,
    pub log_queue: VecDeque<u16>,
    pub halted: bool,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            ram: Ram::default(),
            registers: Registers::default(),
            pc: Wrapping(0),
            sp: Wrapping(0xffff),
            ex: 0,
            ia: 0,
            wait: 0,
            on_decode_error: OnDecodeError::Continue,
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
        self.ram.copy(data.iter(), offset);
    }

    pub fn load_ops(&mut self, ops: &[Instruction<u16>], mut offset: u16) {
        for op in ops {
            offset += op.encode(&mut self.ram[offset..]);
        }
    }

    pub fn get_str(&self, address: u16) -> String {
        use std::char::from_u32;

        let mut msg = String::new();
        for i in address..0xffff {
            if self.ram[i] == 0 {
                break;
            }
            if let Some(c) = from_u32(self.ram[i] as u32) {
                msg.push(c);
            } else {
                break;
            }
        }
        msg
    }

    fn get(&mut self, i: Value<u16>) -> u16 {
        match i {
            Reg(r) => self.registers[r],
            AtReg(r) => self.ram[self.registers[r]],
            AtRegPlus(r, off) => {
                let i = off.wrapping_add(self.get(Reg(r)));
                self.ram[i]
            }
            Push => {
                let v = self.ram[self.sp];
                self.sp += Wrapping(1);
                v
            },
            Peek => self.ram[self.sp],
            Pick(n) => self.ram[self.sp + Wrapping(n)],
            SP => self.sp.0,
            PC => self.pc.0,
            EX => self.ex,
            AtAddr(off) => self.ram[off],
            Litteral(n) => n
        }
    }

    fn set(&mut self, i: Value<u16>, val: u16) {
        match i {
            Reg(r) => self.registers[r] = val,
            AtReg(r) => self.ram[self.registers[r]] = val,
            AtRegPlus(r, off) => {
                let i = off.wrapping_add(self.get(Reg(r)));
                self.ram[i] = val;
            }
            Push => {
                self.sp -= Wrapping(1);
                self.ram[self.sp] = val;
            },
            Peek => self.ram[self.sp] = val,
            Pick(n) => self.ram[self.sp + Wrapping(n)] = val,
            SP => self.sp = Wrapping(val),
            PC => self.pc = Wrapping(val),
            EX => self.ex = val,
            AtAddr(off) => self.ram[off] = val,
            Litteral(_) => ()
        }
    }

    pub fn tick(&mut self, devices: &mut [Box<Device>]) -> Result<CpuState> {
        if self.halted {
            return Err(ErrorKind::Halted.into());
        }
        if self.wait != 0 {
            self.wait -= 1;
            trace!("Waiting");
            return Ok(CpuState::Waiting);
        }

        if !self.is_queue_enabled {
            if let Some(interrupt) = self.interrupts_queue.pop_front() {
                self.exec_interrupt(interrupt);
            }
        }

        let pc = self.pc;
        let (words_used, instruction) = match self.decode(pc.0) {
            Ok(res) => res,
            Err(e) => match self.on_decode_error {
                OnDecodeError::Continue => {
                    warn!("Instruction decoding error: {:x}", self.ram[self.pc]);
                    self.pc += Wrapping(1);
                    return Ok(CpuState::Executing);
                }
                OnDecodeError::Fail => {
                    return Err(e.into());
                }
            }
        };
        self.pc += Wrapping(words_used);

        debug!("Executing {}", instruction);
        // BRK and HLT have a 0 delay
        self.wait = max(instruction.delay(), 1) - 1;
        try!(self.op(instruction, devices));

        Ok(CpuState::Executing)
    }

    fn decode(&mut self, offset: u16) -> Result<(u16, Instruction<u16>)> {
        let bin = [
            self.ram[offset],
            self.ram[offset.wrapping_add(1)],
            self.ram[offset.wrapping_add(2)],
        ];
        Ok(try!(Instruction::decode(&bin)))
    }

    fn exec_interrupt(&mut self, i: u16) {
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

    pub fn hardware_interrupt(&mut self, msg: u16) {
        self.interrupts_queue.push_back(msg);
    }

    fn op(&mut self, i: Instruction<u16>, devices: &mut [Box<Device>]) -> Result<()> {
        match i {
            Instruction::BasicOp(op, b, a) => self.basic_op(op, b, a),
            Instruction::SpecialOp(op, a) => self.special_op(op, a, devices)
        }
    }

    fn basic_op(&mut self, op: BasicOp, b: Value<u16>, a: Value<u16>) -> Result<()> {
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

    fn special_op(&mut self, op: SpecialOp, a: Value<u16>, devices: &mut [Box<Device>]) -> Result<()> {
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

    fn op_set(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.set(b, val_a);
        Ok(())
    }

    fn op_add(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow) = val_b.overflowing_add(val_a);
        self.set(b, new_b);
        self.ex = overflow as u16;
        Ok(())
    }

    fn op_sub(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow) = val_b.overflowing_sub(val_a);
        self.set(b, new_b);
        self.ex = if overflow {0xffff} else {0};
        Ok(())
    }

    fn op_mul(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as u32;
        let val_b = self.get(b) as u32;
        let new_b = val_a * val_b;
        self.set(b, new_b as u16);
        self.ex = (new_b >> 16) as u16;
        Ok(())
    }

    fn op_mli(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as i16 as i32;
        let val_b = self.get(b) as i16 as i32;
        let new_b = (val_a * val_b) as u32;
        self.set(b, new_b as u16);
        self.ex = (new_b >> 16) as u16;
        Ok(())
    }

    fn op_div(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        if val_a == 0 {
            self.set(b, 0);
            self.ex = 0;
        } else {
            let val_b = self.get(b);
            self.set(b, val_b / val_a);
            self.ex = (((val_b as u32) << 16) / (val_a as u32)) as u16;
        }
        Ok(())
    }

    fn op_dvi(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as i16;
        if val_a == 0 {
            self.set(b, 0);
            self.ex = 0;
        } else {
            let val_b = self.get(b) as i16;
            self.set(b, (val_b / val_a) as u16);
            self.ex = (((val_b as i32) << 16) / (val_a as i32)) as u16;
        }
        Ok(())
    }

    fn op_mod(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        if val_a == 0 {
            self.set(b, 0);
        } else {
            let val_b = self.get(b);
            self.set(b, val_b % val_a);
        }
        Ok(())
    }

    fn op_mdi(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as i16;
        if val_a == 0 {
            self.set(b, 0);
        } else {
            let val_b = self.get(b) as i16;
            self.set(b, (val_b % val_a) as u16);
        }
        Ok(())
    }

    fn op_and(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b & val_a);
        Ok(())
    }

    fn op_bor(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b | val_a);
        Ok(())
    }

    fn op_xor(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b ^ val_a);
        Ok(())
    }

    fn op_shr(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b >> val_a);
        self.ex = (((val_b as u32) << 16) >> val_a) as u16;
        Ok(())
    }

    fn op_asr(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b) as i16;
        self.set(b, (val_b >> val_a) as u16);
        self.ex = (((val_b as i32) << 16) >> val_a) as u16;
        Ok(())
    }

    fn op_shl(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.set(b, val_b << val_a);
        self.ex = (((val_b as u32) << val_a) >> 16) as u16;
        Ok(())
    }

    fn exec_if(&mut self, cond: bool) -> Result<()> {
        if !cond {
            self.wait += 1;

            loop {
                let pc = self.pc;
                let (offset, op) = try!(self.decode(pc.0));
                self.pc += Wrapping(offset);

                if op.is_if() {
                    trace!("Skipping cascade");
                    self.wait += 1;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    fn op_ifb(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if((val_b & val_a) != 0)
    }

    fn op_ifc(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if((val_b & val_a) == 0)
    }

    fn op_ife(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_b == val_a)
    }

    fn op_ifn(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_b != val_a)
    }

    fn op_ifg(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_b > val_a)
    }

    fn op_ifa(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as i16;
        let val_b = self.get(b) as i16;
        self.exec_if(val_b > val_a)
    }

    fn op_ifl(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        self.exec_if(val_b < val_a)
    }

    fn op_ifu(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a) as i16;
        let val_b = self.get(b) as i16;
        self.exec_if(val_b < val_a)
    }

    fn op_adx(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow1) = val_b.overflowing_add(val_a);
        let (new_b, overflow2) = new_b.overflowing_add(self.ex);
        if overflow1 || overflow2 {
            self.ex = 1;
        } else {
            self.ex = 0;
        }
        self.set(b, new_b);
        Ok(())
    }

    fn op_sbx(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let val_b = self.get(b);
        let (new_b, overflow1) = val_b.overflowing_sub(val_a);
        let (new_b, overflow2) = new_b.overflowing_add(self.ex);
        if overflow1 || overflow2 {
            self.ex = 0xffff;
        } else {
            self.ex = 0;
        }
        self.set(b, new_b);
        Ok(())
    }

    fn op_sti(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.set(b, val_a);
        self.registers[Register::I] =
            self.registers[Register::I].wrapping_add(1);
        self.registers[Register::J] =
            self.registers[Register::J].wrapping_add(1);
        Ok(())
    }

    fn op_std(&mut self, b: Value<u16>, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.set(b, val_a);
        self.registers[Register::I] =
            self.registers[Register::I].wrapping_sub(1);
        self.registers[Register::J] =
            self.registers[Register::J].wrapping_sub(1);
        Ok(())
    }

    fn op_jsr(&mut self, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        let new_pc = Litteral(self.pc.0);
        try!(self.op_set(Push, new_pc));
        self.pc = Wrapping(val_a);
        Ok(())
    }

    fn op_int(&mut self, a: Value<u16>) -> Result<()> {
        if self.ia != 0 {
            if self.interrupts_queue.len() >= 256 {
                return Err(ErrorKind::InFire.into());
            }
            let val_a = self.get(a);
            self.interrupts_queue.push_back(val_a);
        }
        Ok(())
    }

    fn op_iag(&mut self, a: Value<u16>) -> Result<()> {
        let ia = self.ia;
        self.set(a, ia);
        Ok(())
    }

    fn op_ias(&mut self, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.ia = val_a;
        Ok(())
    }

    fn op_rfi(&mut self, _: Value<u16>) -> Result<()> {
        self.is_queue_enabled = false;
        let v1 = self.get(Push);
        self.set(Reg(Register::A), v1);
        let v2 = self.get(Push);
        self.set(PC, v2);
        Ok(())
    }

    fn op_iaq(&mut self, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.is_queue_enabled = val_a == 0;
        Ok(())
    }

    fn op_hwn(&mut self, a: Value<u16>, devices: &mut [Box<Device>]) -> Result<()> {
        let nb_devices = devices.len();
        self.set(a, nb_devices as u16);
        Ok(())
    }

    fn op_hwq(&mut self, a: Value<u16>, devices: &mut [Box<Device>]) -> Result<()> {
        let val_a = self.get(a) as usize;

        if let Some(device) = devices.get(val_a) {
            let id = device.hardware_id();
            let version = device.hardware_version();
            let manufacturer = device.manufacturer();

            self.set(Reg(Register::A), id as u16);
            self.set(Reg(Register::B), (id >> 16) as u16);
            self.set(Reg(Register::C), version);
            self.set(Reg(Register::X), manufacturer as u16);
            self.set(Reg(Register::Y), (manufacturer >> 16) as u16);
            Ok(())
        } else {
            Err(ErrorKind::InvalidHardwareId(val_a as u16).into())
        }
    }

    fn op_hwi(&mut self, a: Value<u16>, devices: &mut [Box<Device>]) -> Result<()> {
        let val_a = self.get(a) as usize;

        if let Some(device) = devices.get_mut(val_a) {
            self.wait += try!(device.interrupt(self));
            Ok(())
        } else {
            Err(ErrorKind::InvalidHardwareId(val_a as u16).into())
        }
    }

    fn op_log(&mut self, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        self.log_queue.push_back(val_a);
        Ok(())
    }

    fn op_brk(&mut self, a: Value<u16>) -> Result<()> {
        let val_a = self.get(a);
        Err(ErrorKind::Break(val_a).into())
    }

    fn op_hlt(&mut self) -> Result<()> {
        self.halted = true;
        Err(ErrorKind::Halted.into())
    }
}
