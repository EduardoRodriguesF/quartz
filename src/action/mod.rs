use crate::action;
use crate::{cli::Cmd, Ctx};
use crate::{endpoint::EndpointInput, QuartzResult};

pub mod body;
pub mod config;
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

        Cmd::Send {
            header,
            query,
            var,
            request,
            data,
            no_follow,
        } => {
            action::send::cmd(
                ctx,
                action::send::Args {
                    headers: header,
                    query,
                    variables: var,
                    request,
                    data,
                    no_follow,
                },
            )
            .await?
        }

        Cmd::Create {
            handle,
            url,
            method,
            query,
            header: headers,
            switch,
        } => action::handle::create(action::handle::CreateArgs {
            handle,
            config: EndpointInput {
                url,
                method,
                query,
                headers,
                ..Default::default()
            },
            switch,
        }),

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

        Cmd::Ls { depth } => action::ls::cmd(&ctx, depth),
        Cmd::Show { command } => action::show::cmd(&ctx, command)?,
        Cmd::Edit { editor } => action::handle::edit(ctx, action::handle::EditArgs { editor })?,
        Cmd::Cp { src, dest } => action::handle::cp(ctx, action::handle::CpArgs { src, dest }),

        Cmd::Rm { handle, recursive } => {
            action::handle::rm(ctx, action::handle::RmArgs { handle, recursive })
        }

        Cmd::Query { command } => action::query::cmd(&ctx, command)?,
        Cmd::Header { command } => action::header::cmd(&ctx, command)?,

        Cmd::Body { stdin, edit, print } => {
            action::body::cmd(ctx, action::body::BodyArgs { stdin, edit, print })?
        }

        Cmd::Last { command, date } => {
            action::last::cmd(command, action::last::Args { date_format: date })?
        }

        Cmd::History {
            max_count,
            date,
            show,
        } => action::history::cmd(max_count, date, show)?,

        Cmd::Var { command } => action::var::cmd(ctx, command)?,
        Cmd::Context { command } => action::env::cmd(ctx, command)?,
        Cmd::Config { command } => action::config::cmd(ctx, command)?,
    };

    Ok(())
}
