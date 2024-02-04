use crate::{action, cli::ShowCmd as Cmd, Ctx, QuartzResult, StateField};

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Query { key } => {
            if let Some(key) = key {
                action::query::get(ctx, key);
            } else {
                action::query::print(ctx);
            }
        }
        Cmd::Headers { key } => {
            if let Some(key) = key {
                action::header::get(ctx, key);
            } else {
                action::header::ls(ctx);
            }
        }
        Cmd::Url => url(ctx),
        Cmd::Method => method(ctx),
        Cmd::Body => action::body::print(ctx),
        Cmd::Handle => handle(ctx),
        Cmd::Env => action::env::print(ctx),
        Cmd::Endpoint => endpoint(ctx)?,
        Cmd::Snippet { command, var } => action::snippet::cmd(&ctx, command, var)?,
    };

    Ok(())
}

pub fn url(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.url);
}

pub fn method(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.method);
}

pub fn handle(ctx: &Ctx) {
    if let Ok(endpoint) = ctx.state.get(ctx, StateField::Endpoint) {
        println!("{}", endpoint);
    }
}

pub fn endpoint(ctx: &Ctx) -> QuartzResult {
    let (_, endpoint) = ctx.require_endpoint();

    println!("{}", endpoint.to_toml()?);
    Ok(())
}
