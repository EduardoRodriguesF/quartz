use std::process::{exit, ExitCode};

use clap::Parser;
use colored::Colorize;

use quartz_cli::{
    action,
    cli::{Cli, Cmd},
    Ctx, CtxArgs, QuartzResult,
};

#[tokio::main]
async fn main() -> QuartzResult<ExitCode> {
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            info.to_string()
        };

        eprintln!("{}: {payload}", "error".red().bold());
    }));

    let args = Cli::parse();

    // Has to run outside action flow because it cannot resolve `ctx`.
    if let Cmd::Init(args) = args.command {
        action::init::cmd(args)?;
        return Ok(ExitCode::SUCCESS);
    }

    let mut ctx = Ctx::new(CtxArgs {
        from_handle: args.from_handle,
        early_apply_environment: args.apply_environment,
    })?;

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(ctx.config.ui.colors);

    action::cmd(&mut ctx, args.command).await?;

    Ok(*ctx.exit_code())
}
