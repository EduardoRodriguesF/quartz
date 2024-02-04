use std::process::exit;

use crate::{cli::VarCmd as Cmd, env::Variables, Ctx, PairMap, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Edit => edit(ctx)?,
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { variable } => set(ctx, variable)?,
        Cmd::Rm { key } => rm(ctx, key)?,
        Cmd::Ls => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, key: String) {
    let env = ctx.require_env();
    let v = env
        .variables
        .get(&key)
        .unwrap_or_else(|| panic!("{} variable not set", key));

    println!("{}", v);
}

pub fn set(ctx: &Ctx, variables: Vec<String>) -> QuartzResult {
    let mut env = ctx.require_env();
    for input in variables {
        env.variables.set(&input);
    }

    env.update(ctx)?;
    Ok(())
}

pub fn ls(ctx: &Ctx) {
    let env = ctx.require_env();
    print!("{}", env.variables);
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    let env = ctx.require_env();
    ctx.edit(&env.dir(ctx).join("variables"), |c| {
        Variables::parse(c);
        Ok(())
    })?;

    Ok(())
}

pub fn rm(ctx: &Ctx, keys: Vec<String>) -> QuartzResult {
    let mut exit_code = 0;
    let mut env = ctx.require_env();

    for key in keys {
        env.variables.remove(&key).unwrap_or_else(|| {
            exit_code = 1;
            eprintln!("{} variable not set", key);
            "".to_string()
        });
    }

    env.update(ctx)?;
    exit(exit_code);
}
