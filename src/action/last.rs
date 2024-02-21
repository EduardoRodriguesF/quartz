use std::convert::Infallible;

use crate::{
    cli::LastCmd as Cmd,
    cli::LastReqCmd as ReqCmd,
    cli::LastResCmd as ResCmd,
    history::{self, History},
    Ctx, QuartzResult,
};

pub struct Args {
    pub date_format: Option<String>,
}

pub fn cmd(ctx: &Ctx, maybe_command: Option<Cmd>, args: Args) -> QuartzResult<(), Infallible> {
    let mut entry = History::last(ctx).expect("no history found");

    if maybe_command.is_none() {
        println!("{entry}");
        return Ok(());
    }

    if let Some(command) = maybe_command {
        match command {
            Cmd::Handle => println!("{}", entry.handle()),
            Cmd::Req { command } => req(command, &entry),
            Cmd::Res { command } => res(command, &entry),
        }
    };

    Ok(())
}

pub fn req(command: ReqCmd, entry: &history::Entry) {}

pub fn res(command: ResCmd, entry: &history::Entry) {}
