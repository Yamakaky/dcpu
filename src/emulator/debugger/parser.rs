use nom::*;

use assembler::parser::pos_number;

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Step,
    PrintRegisters,
    Disassemble {
        from: u16,
        size: u16,
    },
    Examine {
        from: u16,
        size: u16,
    },
    Breakpoint(u16),
    Continue,
    ShowBreakpoints,
    DeleteBreakpoint(u16),
    ShowDevices,
}

named!(pub parse_command<Command>,
    delimited!(
        opt!(multispace),
        alt_complete!(
            cmd_step |
            cmd_print_regs |
            cmd_disassemble |
            cmd_examine |
            cmd_breakpoint |
            cmd_continue |
            cmd_show_breakpoints |
            cmd_delete_breakpoint |
            cmd_show_devices
        ),
        opt!(multispace)
    )
);

named!(cmd_step<Command>,
    map!(char!('s'), |_| Command::Step)
);

named!(cmd_print_regs<Command>,
    map!(char!('r'), |_| Command::PrintRegisters)
);

named!(cmd_disassemble<Command>,
    chain!(tag!("disassemble") ~
           multispace ~
           from: pos_number ~
           multispace ~
           size: pos_number,
           || Command::Disassemble{from: from, size: size}
    )
);

named!(cmd_examine<Command>,
    chain!(char!('x') ~
           multispace ~
           from: pos_number ~
           multispace ~
           size: pos_number,
           || Command::Examine{from: from, size: size}
    )
);

named!(cmd_breakpoint<Command>,
    chain!(
        tag!("b") ~
        multispace ~
        addr: pos_number,
        || Command::Breakpoint(addr)
    )
);

named!(cmd_show_breakpoints<Command>,
    map!(tag!("breakpoints"), |_| Command::ShowBreakpoints)
);

named!(cmd_delete_breakpoint<Command>,
    chain!(
        tag!("delete") ~
        multispace ~
        addr: pos_number,
        || Command::DeleteBreakpoint(addr)
    )
);

named!(cmd_continue<Command>,
    map!(char!('c'), |_| Command::Continue)
);

named!(cmd_show_devices<Command>,
    map!(tag!("devices"), |_| Command::ShowDevices)
);
