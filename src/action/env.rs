use crate::{cli::EnvCmd as Cmd, Ctx, Env, QuartzResult, StateField};
use colored::Colorize;

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Create { name } => create(ctx, name),
        Cmd::Cp { src, dest } => cp(ctx, src, dest)?,
        Cmd::Use { env } => switch(ctx, env)?,
        Cmd::Ls => ls(ctx),
        Cmd::Rm { env } => rm(ctx, env),
    };

    Ok(())
}

pub fn create(ctx: &Ctx, name: String) {
    let env = Env::new(&name);

    if env.exists(ctx) {
        panic!("a environment named {} already exists", name.red());
    }

    if env.write(ctx).is_err() {
        panic!("failed to create {} environment", name);
    }
}

pub fn cp(ctx: &Ctx, src: String, dest: String) -> QuartzResult {
    let src = Env::parse(ctx, &src).unwrap_or_else(|_| {
        panic!("no {} environment found", &src);
    });
    let mut dest = Env::parse(ctx, &dest).unwrap_or(Env::new(&dest));

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

pub fn switch(ctx: &Ctx, env: String) -> QuartzResult {
    let env = Env::new(&env);

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

pub fn rm(ctx: &Ctx, env: String) {
    let env = Env::new(&env);

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
