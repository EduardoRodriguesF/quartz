use crate::{cli::VarCmd as Cmd, env::Variables, Ctx, PairMap, QuartzResult};
use std::process::ExitCode;

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    #[arg(name = "VARIABLE", required = true)]
    variables: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    #[arg(name = "KEY", required = true)]
    keys: Vec<String>,
}

pub fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Edit => edit(ctx)?,
        Cmd::Get(args) => get(ctx, args),
        Cmd::Set(args) => set(ctx, args)?,
        Cmd::Rm(args) => rm(ctx, args)?,
        Cmd::Ls => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, args: GetArgs) {
    let env = ctx.require_env();
    let v = env
        .variables
        .get(&args.key)
        .unwrap_or_else(|| panic!("{} variable not set", args.key));

    println!("{}", v);
}

pub fn set(ctx: &Ctx, args: SetArgs) -> QuartzResult {
    let mut env = ctx.require_env();
    for input in args.variables {
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

pub fn rm(ctx: &mut Ctx, args: RmArgs) -> QuartzResult {
    let mut env = ctx.require_env();

    for key in args.keys {
        env.variables.remove(&key).unwrap_or_else(|| {
            ctx.code(ExitCode::FAILURE);
            eprintln!("{}: No such variable", key);
            "".to_string()
        });
    }

    env.update(ctx)?;
    Ok(())
}
