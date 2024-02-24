use crate::action;
use crate::{cli::Cmd, Ctx};
use crate::{endpoint::EndpointInput, QuartzResult};

pub mod body;
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
        Cmd::Init { directory } => action::init::cmd(directory)?,

        Cmd::Send(args) => action::send::cmd(ctx, args).await?,

        Cmd::Create {
            handle,
            url,
            method,
            query,
            header: headers,
            switch,
        } => action::handle::create(
            ctx,
            action::handle::CreateArgs {
                handle,
                config: EndpointInput {
                    url,
                    method,
                    query,
                    headers,
                    ..Default::default()
                },
                switch,
            },
        ),

        Cmd::Use {
            handle,
            url,
            method,
            query,
            header: headers,
            empty,
        } => action::handle::switch(
            ctx,
            handle::SwitchArgs {
                handle,
                config: EndpointInput {
                    url,
                    method,
                    query,
                    headers,
                    ..Default::default()
                },
                empty,
            },
        ),

        Cmd::Ls(args) => action::ls::cmd(&ctx, args),
        Cmd::Show { command } => action::show::cmd(&ctx, command)?,
        Cmd::Edit(args) => action::handle::edit(ctx, args)?,
        Cmd::Cp(args) => action::handle::cp(ctx, args)?,
        Cmd::Mv(args) => action::handle::mv(ctx, args)?,
        Cmd::Rm(args) => action::handle::rm(ctx, args)?,

        Cmd::Query { command } => action::query::cmd(ctx, command)?,
        Cmd::Header { command } => action::header::cmd(ctx, command)?,

        Cmd::Body { format, command } => {
            action::body::cmd(ctx, command, action::body::BodyArgs { format })?
        }

        Cmd::Last { command } => action::last::cmd(ctx, command)?,

        Cmd::History { max_count } => {
            action::history::cmd(ctx, action::history::Args { max_count })?
        }

        Cmd::Var { command } => action::var::cmd(ctx, command)?,
        Cmd::Env { command } => action::env::cmd(ctx, command)?,
        Cmd::Config { command } => action::config::cmd(ctx, command)?,
    };

    Ok(())
}
