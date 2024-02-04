use clap::Parser;
use colored::Colorize;

use quartz_cli::{action, cli::Cli, Ctx, CtxArgs, QuartzResult};

#[tokio::main]
async fn main() -> QuartzResult {
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
    let mut ctx = Ctx::new(CtxArgs {
        from_handle: args.from_handle,
        early_apply_context: args.apply_context,
    })?;

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(ctx.config.ui.colors);

    action::cmd(&mut ctx, args.command).await?;
    Ok(())
}
