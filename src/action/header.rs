use std::convert::Infallible;

use crate::{cli::HeaderCmd as Cmd, Ctx, PairMap, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult<(), Infallible> {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { header } => set(ctx, header),
        Cmd::Remove { key } => rm(ctx, key),
        Cmd::List => ls(ctx),
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
    let (handle, mut endpoint) = ctx.require_endpoint();

    for input in headers {
        endpoint.headers.set(&input);
    }

    endpoint.write(&handle);
}

pub fn rm(ctx: &Ctx, keys: Vec<String>) {
    let (handle, mut endpoint) = ctx.require_endpoint();

    for k in keys {
        endpoint.headers.remove(&k);
    }

    endpoint.write(&handle);
}

pub fn ls(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();

    print!("{}", endpoint.headers);
}
