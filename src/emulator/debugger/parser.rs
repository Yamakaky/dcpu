use std::fmt::{Debug, Display};

use clap;
use nom;

pub use assembler::types::Expression;
use assembler::parser::nom_parser::{expression, pos_number};

error_chain! {
    foreign_links {
        clap::Error, Clap;
    }

    errors {
        Nom(e: String) {
            description("parsing error")
            display("parsing error: {}", e)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Step(u16),
    PrintRegisters,
    Disassemble {
        from: Expression,
        size: u16,
    },
    Examine {
        from: Expression,
        size: u16,
    },
    Breakpoint(Expression),
    Continue,
    ShowBreakpoints,
    DeleteBreakpoint(u16),
    ShowDevices,
    Hook(Box<Command>),
    Logs,
    M35fd(u16, M35fdCmd),
    Stack(u16),
    Symbols,
}

#[derive(Debug, Clone)]
pub enum M35fdCmd {
    Eject,
    Load(String),
}

fn clap_parser<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("DCPU debugger")
        .author(crate_authors!())
        .version(crate_version!())
        .setting(clap::AppSettings::VersionlessSubcommands)
        .setting(clap::AppSettings::NoBinaryName)
        .subcommand(clap::SubCommand::with_name("step")
            .visible_alias("s")
            .help("Execute one instruction.")
            .arg(clap::Arg::with_name("count")))
        .subcommand(clap::SubCommand::with_name("registers")
            .visible_alias("r")
            .help("Show the registers."))
        .subcommand(clap::SubCommand::with_name("disassemble")
            .help("Disassemble a memory part.")
            .arg(clap::Arg::with_name("base")
                .help("From where to disassemble (default [PC]).")
                .required(true))
            .arg(clap::Arg::with_name("length")
                .help("Number of instructions to disassemble.")))
        .subcommand(clap::SubCommand::with_name("examine")
            .visible_alias("x")
            .help("Print a memory slice as hexadecimal.")
            .arg(clap::Arg::with_name("base")
                .help("From where to disassemble (default [PC]).")
                .required(true))
            .arg(clap::Arg::with_name("length")
                .help("Number of instructions to disassemble.")))
        .subcommand(clap::SubCommand::with_name("break")
            .visible_alias("b")
            .help("Add a breakpoint.")
            .arg(clap::Arg::with_name("expression")
                .multiple(true)
                .required(true)))
        .subcommand(clap::SubCommand::with_name("continue")
            .visible_alias("c")
            .help("Continue the execution."))
        .subcommand(clap::SubCommand::with_name("breakpoints")
            .help("Show the active breakpoints"))
        .subcommand(clap::SubCommand::with_name("delete")
            .visible_alias("d")
            .help("Delete a breakpoint.")
            .arg(clap::Arg::with_name("id")
                .required(true)))
        .subcommand(clap::SubCommand::with_name("devices")
            .help("Show the connected devices."))
        .subcommand(clap::SubCommand::with_name("hook")
            .help("Run this command before each new prompt.")
            .arg(clap::Arg::with_name("command")
                .multiple(true)
                .required(true)))
        .subcommand(clap::SubCommand::with_name("logs")
            .help("Show the values in the LOG buffer."))
        .subcommand(clap::SubCommand::with_name("m35fd")
            .setting(clap::AppSettings::SubcommandRequired)
            .help("M35FD-specific commands.")
            .arg(clap::Arg::with_name("id")
                .required(true))
            .subcommand(clap::SubCommand::with_name("eject")
                .help("Eject the floppy."))
            .subcommand(clap::SubCommand::with_name("load")
                .help("Load a new floppy.")
                .arg(clap::Arg::with_name("file")
                    .required(true))))
        .subcommand(clap::SubCommand::with_name("stack")
            .help("Show <count> bytes from the stack.")
            .arg(clap::Arg::with_name("count")))
        .subcommand(clap::SubCommand::with_name("symbols")
            .help("Show the symbols."))
}

pub fn parse_command(cmd: &str) -> Result<Command> {
    let raw = try!(clap_parser().get_matches_from_safe(cmd.split(" ")));
    Command::try_from(&raw)
}

impl Command {
    fn try_from(matches: &clap::ArgMatches) -> Result<Command> {
        match matches.subcommand() {
            ("step", Some(val)) => {
                let str_count = val.value_of("count").unwrap_or("1");
                let count = try!(conv_iresult(pos_number(str_count.as_bytes())));
                Ok(Command::Step(count))
            }
            ("registers", _) => Ok(Command::PrintRegisters),
            ("disassemble", Some(args)) => {
                let str_from = args.value_of("base").unwrap();
                let from = try!(conv_iresult(expression(str_from.as_bytes())));
                let str_len = args.value_of("length").unwrap_or("10");
                let len = try!(conv_iresult(pos_number(str_len.as_bytes())));
                Ok(Command::Disassemble {
                    from: from,
                    size: len,
                })
            }
            ("examine", Some(args)) => {
                let str_from = args.value_of("base").unwrap();
                let from = try!(conv_iresult(expression(str_from.as_bytes())));
                let str_len = args.value_of("length").unwrap_or("10");
                let len = try!(conv_iresult(pos_number(str_len.as_bytes())));
                Ok(Command::Examine {
                    from: from,
                    size: len,
                })
            }
            ("break", Some(val)) => {
                let str_expr = val.values_of("expression")
                                  .unwrap()
                                  .collect::<Vec<_>>()
                                  .join(" ");
                let expr = try!(conv_iresult(expression(str_expr.as_bytes())));
                Ok(Command::Breakpoint(expr))
            }
            ("continue", _) => Ok(Command::Continue),
            ("breakpoints", _) => Ok(Command::ShowBreakpoints),
            ("delete", Some(id)) => {
                let str_id = id.value_of("id").unwrap();
                let id = try!(conv_iresult(pos_number(str_id.as_bytes())));
                Ok(Command::DeleteBreakpoint(id))
            }
            ("devices", Some(_)) => Ok(Command::ShowDevices),
            ("hook", Some(cmd)) => {
                let parsed = try!(parse_command(&cmd.values_of("command")
                                                    .unwrap()
                                                    .collect::<Vec<_>>()
                                                    .join(" ")));
                Ok(Command::Hook(Box::new(parsed)))
            }
            ("logs", Some(_)) => Ok(Command::Logs),
            ("m35fd", Some(args)) => {
                let str_id = args.value_of("id").unwrap();
                let id = try!(conv_iresult(pos_number(str_id.as_bytes())));
                match args.subcommand() {
                    ("eject", Some(_)) =>
                        Ok(Command::M35fd(id, M35fdCmd::Eject)),
                    ("load", Some(args)) => {
                        let file = args.value_of("file").unwrap();
                        Ok(Command::M35fd(id, M35fdCmd::Load(file.into())))
                    }
                    _ => unreachable!(),
                }
            }
            ("stack", Some(args)) => {
                let str_count = args.value_of("count").unwrap_or("10");
                let count = try!(conv_iresult(pos_number(str_count.as_bytes())));
                Ok(Command::Stack(count))
            }
            ("symbols", Some(_)) => Ok(Command::Symbols),
            (cmd, args) => {
                try!(Err(format!("unknown command \"{}\" ({:?})", cmd, args)))
            }
        }
    }
}

fn conv_iresult<O: Display + Debug>(ires: nom::IResult<&[u8], O>) -> Result<O> {
    use nom::IResult;

    match ires {
        IResult::Done(i, o) => {
            if i.len() == 0 {
                Ok(o)
            } else {
                try!(Err(format!("garbage after {}", o)))
            }
        }
        IResult::Error(e) => try!(Err(ErrorKind::Nom(format!("{}", e)))),
        IResult::Incomplete(_) => unreachable!(),
    }
}
