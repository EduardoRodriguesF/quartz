use crate::{cli::HeaderCmd as Cmd, Ctx, PairMap, QuartzResult};
use std::process::ExitCode;

pub fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { header } => set(ctx, header),
        Cmd::Rm { key } => rm(ctx, key),
        Cmd::Ls => ls(ctx),
    }
}

pub fn get(ctx: &Ctx, key: String) -> QuartzResult {
    let (_, endpoint) = ctx.require_endpoint();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }

    Ok(())
}

pub fn set(ctx: &Ctx, headers: Vec<String>) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();

    for input in headers {
        endpoint.headers.set(&input);
    }

    endpoint.write();
    Ok(())
}

pub fn rm(ctx: &mut Ctx, keys: Vec<String>) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();

    for k in keys {
        if endpoint.headers.contains_key(&k) {
            endpoint.headers.remove(&k);
            println!("Removed header: {}", k);
        } else {
            ctx.code(ExitCode::FAILURE);
            eprintln!("{}: No such header", k);
        }
    }

    endpoint.write();
    Ok(())
}

pub fn ls(ctx: &Ctx) -> QuartzResult {
    let (_, endpoint) = ctx.require_endpoint();

    print!("{}", endpoint.headers);
    Ok(())
}
