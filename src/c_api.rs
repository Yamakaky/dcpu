use emulator::{Debugger};
use emulator::cpu::{Cpu, OnDecodeError};
use types::Register;

#[repr(C)]
pub struct CRegisters {
    a: u16,
    b: u16,
    c: u16,
    i: u16,
    j: u16,
    x: u16,
    y: u16,
    z: u16,
    pc: u16,
    ia: u16,
    sp: u16,
    ex: u16,
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_new() -> *mut Debugger {
    let cpu = Cpu::new(OnDecodeError::Fail);
    let devices = vec![];
    let d = Box::new(Debugger::new(cpu, devices));
    Box::into_raw(d)
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_ram(d: *mut Debugger) -> *mut u16 {
    (*d).cpu.ram.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_registers(d: *mut Debugger) -> CRegisters {
    let d = &*d;
    CRegisters {
        pc: d.cpu.pc.0,
        ia: d.cpu.ia,
        sp: d.cpu.sp.0,
        ex: d.cpu.ex,
        a: d.cpu.registers[Register::A],
        b: d.cpu.registers[Register::B],
        c: d.cpu.registers[Register::C],
        i: d.cpu.registers[Register::I],
        j: d.cpu.registers[Register::J],
        x: d.cpu.registers[Register::X],
        y: d.cpu.registers[Register::Y],
        z: d.cpu.registers[Register::Z],
    }
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_step(d: *mut Debugger) {
    // TODO: better error handling
    let _ = (*d).step();
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_continue(d: *mut Debugger) {
    // TODO: better error handling
    let _ = (*d).continue_exec();
}

#[no_mangle]
pub unsafe extern fn dcpu_debugger_free(d: *mut Debugger) {
    // For drop()
    let _ = Box::from_raw(d);
}
