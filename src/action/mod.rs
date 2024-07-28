use crate::action;
use crate::QuartzResult;
use crate::{cli::Cmd, Ctx};

pub mod body;
pub mod completion;
pub mod config;
pub mod cookie;
pub mod env;
pub mod handle;
pub mod header;
pub mod history;
pub mod init;
pub mod last;
pub mod ls;
pub mod query;
pub mod send;
pub mod show;
pub mod snippet;
pub mod var;

pub async fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(ctx, args).await?,
        Cmd::Create(args) => action::handle::create(ctx, args),
        Cmd::Use(args) => action::handle::switch(ctx, args),
        Cmd::Ls(args) => action::ls::cmd(ctx, args),
        Cmd::Show { command } => action::show::cmd(ctx, command)?,
        Cmd::Edit => action::handle::edit(ctx)?,
        Cmd::Cp(args) => action::handle::cp(ctx, args)?,
        Cmd::Mv(args) => action::handle::mv(ctx, args)?,
        Cmd::Rm(args) => action::handle::rm(ctx, args)?,
        Cmd::Query { command } => action::query::cmd(ctx, command)?,
        Cmd::Header { command } => action::header::cmd(ctx, command)?,
        Cmd::Body(args) => action::body::cmd(ctx, args)?,
        Cmd::History(args) => action::history::cmd(ctx, args)?,
        Cmd::Last { command } => action::last::cmd(ctx, command)?,
        Cmd::Var { command } => action::var::cmd(ctx, command)?,
        Cmd::Env { command } => action::env::cmd(ctx, command)?,
        Cmd::Config { command } => action::config::cmd(ctx, command)?,
        Cmd::Completion(args) => action::completion::cmd(args),
    };

    Ok(())
}
