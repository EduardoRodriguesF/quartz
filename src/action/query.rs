use crate::{cli::QueryCmd as Cmd, Ctx, PairMap, QuartzResult};
use colored::Colorize;
use std::convert::Infallible;

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult<(), Infallible> {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { query } => set(ctx, query),
        Cmd::Remove { key } => rm(ctx, key),
        Cmd::List => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, key: String) {
    let (_, endpoint) = ctx.require_endpoint();

    let value = endpoint
        .query
        .get(&key)
        .unwrap_or_else(|| panic!("no query param {} found", key.red()));

    println!("{value}");
}

pub fn set(ctx: &Ctx, queries: Vec<String>) {
    let (handle, mut endpoint) = ctx.require_endpoint();

    for input in queries {
        endpoint.query.set(&input);
    }

    endpoint.write(&handle);
}

pub fn rm(ctx: &Ctx, key: String) {
    let (handle, mut endpoint) = ctx.require_endpoint();

    endpoint.query.remove(&key);

    endpoint.write(&handle);
}

pub fn ls(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    print!("{}", endpoint.query);
}

pub fn print(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.query_string());
}