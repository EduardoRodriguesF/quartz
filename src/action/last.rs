use std::convert::Infallible;

use crate::{
    cli::LastCmd as Cmd,
    cli::LastReqCmd as ReqCmd,
    cli::LastResCmd as ResCmd,
    history::{self, History},
    QuartzResult,
};

pub struct Args {
    pub date_format: Option<String>,
}

pub fn cmd(maybe_command: Option<Cmd>, args: Args) -> QuartzResult<(), Infallible> {
    let mut entry = History::last().expect("no history found");

    if let Some(format) = args.date_format {
        entry.date_format(format);
    }

    if maybe_command.is_none() {
        println!("{entry}");
        return Ok(());
    }

    if let Some(command) = maybe_command {
        match command {
            Cmd::Handle => println!("{}", entry.handle),
            Cmd::Date => println!("{}", entry.date().unwrap_or("Unknown".into())),
            Cmd::Req { command } => req(command, &entry.request),
            Cmd::Res { command } => res(command, &entry.response),
        }
    };

    Ok(())
}

pub fn req(command: ReqCmd, request: &history::Request) {
    match command {
        ReqCmd::Url => println!("{}", request.endpoint.url),
        ReqCmd::Query => print!("{}", request.endpoint.query),
        ReqCmd::Headers => print!("{}", request.endpoint.headers),
        ReqCmd::Method => println!("{}", request.endpoint.method),
        ReqCmd::Body => print!("{}", request.body),
        ReqCmd::Context => println!("{}", request.context.name),
    }
}

pub fn res(command: ResCmd, response: &history::Response) {
    match command {
        ResCmd::Status => println!("{}", response.status),
        ResCmd::Headers => print!("{}", response.headers),
        ResCmd::Body => print!("{}", response.body),
        ResCmd::Size => println!("{}", response.size),
    }
}
