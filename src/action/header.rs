use std::{convert::Infallible, process::exit};

use crate::{cli::HeaderCmd as Cmd, Ctx, PairMap, QuartzExitCode, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult<(), Infallible> {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { header } => set(ctx, header),
        Cmd::Rm { key } => rm(ctx, key),
        Cmd::Ls => ls(ctx),
    }

    Ok(())
}

pub fn get(ctx: &Ctx, key: String) {
    let (_, endpoint) = ctx.require_endpoint();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }
}

pub fn set(ctx: &Ctx, headers: Vec<String>) {
    let (_, mut endpoint) = ctx.require_endpoint();

    for input in headers {
        endpoint.headers.set(&input);
    }

    endpoint.write();
}

pub fn rm(ctx: &Ctx, keys: Vec<String>) {
    let mut code = QuartzExitCode::Success;
    let (_, mut endpoint) = ctx.require_endpoint();

    for k in keys {
        if endpoint.headers.contains_key(&k) {
            endpoint.headers.remove(&k);
            println!("Removed header: {}", k);
        } else {
            code = QuartzExitCode::Error;
            eprintln!("{}: No such header", k);
        }
    }

    endpoint.write();
    exit(code as i32);
}

pub fn ls(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();

    print!("{}", endpoint.headers);
}
