use std::convert::Infallible;

use crate::{
    cli::LastCmd as Cmd,
    cli::LastResCmd as ResCmd,
    history::{self, History},
    Ctx, QuartzResult,
};

pub fn cmd(ctx: &Ctx, maybe_command: Option<Cmd>) -> QuartzResult<(), Infallible> {
    let entry = History::last(ctx).expect("no history found");

    if maybe_command.is_none() {
        println!("{entry}");
        return Ok(());
    }

    if let Some(command) = maybe_command {
        match command {
            Cmd::Handle => println!("{}", entry.handle()),
            Cmd::Req => req(&entry),
            Cmd::Res { command } => res(command, &entry),
        }
    };

    Ok(())
}

pub fn req(entry: &history::Entry) {
    req_head(entry);
}

pub fn req_head(entry: &history::Entry) {
    let iter = entry
        .messages()
        .iter()
        .filter_map(|p| match p.starts_with(">") {
            true => Some(
                p.split("\n")
                    .map(|s| s.trim_start_matches(">").trim())
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ),
            false => None,
        });

    for m in iter {
        println!("{m}");
    }
}

pub fn res(command: Option<ResCmd>, entry: &history::Entry) {
    if let Some(command) = command {
        match command {
            ResCmd::Head => res_head(entry),
            ResCmd::Body => res_body(entry),
        }
    } else {
        res_head(entry);
        res_body(entry);
    }
}

pub fn res_head(entry: &history::Entry) {
    let iter = entry
        .messages()
        .iter()
        .filter_map(|p| match p.starts_with("<") {
            true => Some(
                p.split("\n")
                    .map(|s| s.trim_start_matches("<").trim())
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ),
            false => None,
        });

    for m in iter {
        println!("{m}");
    }
}

pub fn res_body(entry: &history::Entry) {
    if let Some(body) = entry.messages().last() {
        println!("{}", body);
    }
}
