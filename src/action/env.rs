use crate::{cli::EnvCmd as Cmd, Context, Ctx, QuartzResult, StateField};
use colored::Colorize;
use std::path::Path;

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Create { name } => create(name),
        Cmd::Copy { src, dest } => cp(src, dest)?,
        Cmd::Use { context } => switch(context)?,
        Cmd::List => ls(ctx),
        Cmd::Remove { context } => rm(context),
    };

    Ok(())
}

pub fn create(name: String) {
    let context = Context::new(&name);

    if context.exists() {
        panic!("a context named {} already exists", name.red());
    }

    if context.write().is_err() {
        panic!("failed to create {} context", name);
    }
}

pub fn cp(src: String, dest: String) -> QuartzResult {
    let src = Context::parse(&src).unwrap_or_else(|_| {
        panic!("no {} context found", &src);
    });
    let mut dest = Context::parse(&dest).unwrap_or(Context::new(&dest));

    for (key, value) in src.variables.iter() {
        dest.variables.insert(key.to_string(), value.to_string());
    }

    if dest.exists() {
        dest.update()?;
    } else {
        dest.write()?;
    }

    Ok(())
}

pub fn switch(context: String) -> QuartzResult {
    let context = Context::new(&context);

    if !context.exists() {
        panic!("context {} does not exist", context.name.red());
    }

    if let Ok(()) = StateField::Context.set(&context.name) {
        println!("Switched to {} context", context.name.green());
    } else {
        panic!("failed to switch to {} context", context.name.red());
    }

    Ok(())
}

pub fn ls(ctx: &Ctx) {
    if let Ok(entries) = std::fs::read_dir(Path::new(".quartz").join("contexts")) {
        for entry in entries {
            let bytes = entry.unwrap().file_name();
            let context_name = bytes.to_str().unwrap();

            let state = ctx
                .state
                .get(StateField::Context)
                .unwrap_or(String::from("default"));

            if state == context_name {
                println!("* {}", context_name.green());
            } else {
                println!("  {}", context_name);
            }
        }
    }
}

pub fn rm(context: String) {
    let context = Context::new(&context);

    if !context.exists() {
        panic!("context {} does not exist", context.name.red());
    }

    if std::fs::remove_dir_all(context.dir()).is_ok() {
        println!("Deleted {} context", context.name.green());
    } else {
        panic!("failed to delete {} context", context.name.red());
    }
}

pub fn print(ctx: &Ctx) {
    println!(
        "{}",
        ctx.state
            .get(StateField::Context)
            .unwrap_or("default".into())
    );
}
