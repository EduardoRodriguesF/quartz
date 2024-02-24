use crate::{cli::EnvCmd as Cmd, Ctx, Env, QuartzResult, StateField};
use colored::Colorize;

#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    name: String,
}

#[derive(clap::Args, Debug)]
pub struct CpArgs {
    src: String,
    dest: String,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    env: String,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    env: String,
}

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Create(args) => create(ctx, args),
        Cmd::Cp(args) => cp(ctx, args)?,
        Cmd::Use(args) => switch(ctx, args)?,
        Cmd::Ls => ls(ctx),
        Cmd::Rm(args) => rm(ctx, args),
    };

    Ok(())
}

pub fn create(ctx: &Ctx, args: CreateArgs) {
    let env = Env::new(&args.name);

    if env.exists(ctx) {
        panic!("a environment named {} already exists", args.name.red());
    }

    if env.write(ctx).is_err() {
        panic!("failed to create {} environment", args.name);
    }
}

pub fn cp(ctx: &Ctx, args: CpArgs) -> QuartzResult {
    let src = Env::parse(ctx, &args.src).unwrap_or_else(|_| {
        panic!("no {} environment found", &args.src);
    });
    let mut dest = Env::parse(ctx, &args.dest).unwrap_or(Env::new(&args.dest));

    for (key, value) in src.variables.iter() {
        dest.variables.insert(key.to_string(), value.to_string());
    }

    if dest.exists(ctx) {
        dest.update(ctx)?;
    } else {
        dest.write(ctx)?;
    }

    Ok(())
}

pub fn switch(ctx: &Ctx, args: SwitchArgs) -> QuartzResult {
    let env = Env::new(&args.env);

    if !env.exists(ctx) {
        panic!("environment {} does not exist", env.name.red());
    }

    if let Ok(()) = StateField::Env.set(ctx, &env.name) {
        println!("Switched to {} environment", env.name.green());
    } else {
        panic!("failed to switch to {} environment", env.name.red());
    }

    Ok(())
}

pub fn ls(ctx: &Ctx) {
    if let Ok(entries) = std::fs::read_dir(ctx.path().join("env")) {
        for entry in entries {
            let bytes = entry.unwrap().file_name();
            let env_name = bytes.to_str().unwrap();

            let state = ctx
                .state
                .get(ctx, StateField::Env)
                .unwrap_or(String::from("default"));

            if state == env_name {
                println!("* {}", env_name.green());
            } else {
                println!("  {}", env_name);
            }
        }
    }
}

pub fn rm(ctx: &Ctx, args: RmArgs) {
    let env = Env::new(&args.env);

    if !env.exists(ctx) {
        panic!("environment {} does not exist", env.name.red());
    }

    if std::fs::remove_dir_all(env.dir(ctx)).is_ok() {
        println!("Deleted {} environment", env.name.green());
    } else {
        panic!("failed to delete {} environment", env.name.red());
    }
}

pub fn print(ctx: &Ctx) {
    println!(
        "{}",
        ctx.state
            .get(ctx, StateField::Env)
            .unwrap_or("default".into())
    );
}
