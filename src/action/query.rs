use crate::{cli::QueryCmd as Cmd, Ctx, PairMap, QuartzResult};
use colored::Colorize;
use std::process::ExitCode;

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    #[arg(name = "QUERY", required = true)]
    queries: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    #[arg(name = "QUERY", required = true)]
    keys: Vec<String>,
}

pub fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => get(ctx, args.key),
        Cmd::Set(args) => set(ctx, args.queries),
        Cmd::Rm(args) => rm(ctx, args.keys)?,
        Cmd::Ls => ls(ctx),
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
    let (_, mut endpoint) = ctx.require_endpoint();

    for input in queries {
        endpoint.query.set(&input);
    }

    endpoint.write();
}

pub fn rm(ctx: &mut Ctx, keys: Vec<String>) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();

    for k in keys {
        if endpoint.query.contains_key(&k) {
            endpoint.query.remove(&k);
            println!("Removed query param: {}", k);
        } else {
            ctx.code(ExitCode::FAILURE);
            eprintln!("{}: No such query param", k);
        }
    }

    endpoint.write();
    Ok(())
}

pub fn ls(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    print!("{}", endpoint.query);
}

pub fn print(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.query_string());
}
