use clap;
use nom;

pub use assembler::types::Expression;
use assembler::parser::{expression, pos_number};

error_chain! {
    foreign_links {
        clap::Error, Clap;
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Step(u16),
    PrintRegisters,
    Disassemble {
        from: u16,
        size: u16,
    },
    Examine {
        from: u16,
        size: u16,
    },
    Breakpoint(Expression),
    Continue,
    ShowBreakpoints,
    DeleteBreakpoint(u16),
    ShowDevices,
    Hook(Box<Command>),
    Logs,
    ClockCmd(u16),
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
            .arg(clap::Arg::with_name("count")
                .required(true)))
        .subcommand(clap::SubCommand::with_name("registers")
            .visible_alias("r")
            .help("Show the registers."))
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
        .subcommand(clap::SubCommand::with_name("clock")
            .help("Clock-specific commands")
            .arg(clap::Arg::with_name("id")
                .required(true)))
}

pub fn parse_command(cmd: &str) -> Result<Command> {
    let raw = try!(clap_parser().get_matches_from_safe(cmd.split(" ")));
    Command::try_from(&raw)
}

impl Command {
    fn try_from<'a>(matches: &clap::ArgMatches<'a>) -> Result<Command> {
        match matches.subcommand() {
            ("step", Some(val)) => {
                let str_count = val.value_of("count").unwrap();
                let num = match pos_number(str_count.as_bytes()) {
                    nom::IResult::Done(i, o) if i.len() == 0 => o,
                    _ => try!(Err("nom nom nom")),
                };
                Ok(Command::Step(num))
            }
            ("registers", _) => Ok(Command::PrintRegisters),
            ("break", Some(val)) => {
                let str_expr = val.values_of("expression")
                                  .unwrap()
                                  .collect::<Vec<_>>()
                                  .join(" ");
                let expr = match expression(str_expr.as_bytes()) {
                    nom::IResult::Done(i, ref o) if i.len() == 0 => o.clone(),
                    _ => try!(Err("nom nom nom")),
                };
                Ok(Command::Breakpoint(expr))
            }
            ("continue", _) => Ok(Command::Continue),
            ("breakpoints", _) => Ok(Command::ShowBreakpoints),
            ("delete", Some(id)) => {
                let str_id = id.value_of("id").unwrap();
                let id = match pos_number(str_id.as_bytes()) {
                    nom::IResult::Done(i, o) if i.len() == 0 => o,
                    _ => try!(Err("nom nom nom")),
                };
                Ok(Command::DeleteBreakpoint(id))
            }
            ("devices", Some(_)) => Ok(Command::ShowDevices),
            ("hook", Some(cmd)) => {
                println!("{:?}", cmd);
                Ok(Command::Hook(Box::new(try!(Command::try_from(&cmd)))))
            }
            ("logs", Some(_)) => Ok(Command::Logs),
            ("clock", Some(_)) => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
