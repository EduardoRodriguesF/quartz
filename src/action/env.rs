use crate::{cli::EnvCmd as Cmd, Context, Ctx, QuartzResult, StateField};
use colored::Colorize;

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Create { name } => create(ctx, name),
        Cmd::Cp { src, dest } => cp(ctx, src, dest)?,
        Cmd::Use { context } => switch(ctx, context)?,
        Cmd::Ls => ls(ctx),
        Cmd::Rm { context } => rm(ctx, context),
    };

    Ok(())
}

pub fn create(ctx: &Ctx, name: String) {
    let context = Context::new(&name);

    if context.exists(ctx) {
        panic!("a context named {} already exists", name.red());
    }

    if context.write(ctx).is_err() {
        panic!("failed to create {} context", name);
    }
}

pub fn cp(ctx: &Ctx, src: String, dest: String) -> QuartzResult {
    let src = Context::parse(ctx, &src).unwrap_or_else(|_| {
        panic!("no {} context found", &src);
    });
    let mut dest = Context::parse(ctx, &dest).unwrap_or(Context::new(&dest));

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

pub fn switch(ctx: &Ctx, context: String) -> QuartzResult {
    let context = Context::new(&context);

    if !context.exists(ctx) {
        panic!("context {} does not exist", context.name.red());
    }

    if let Ok(()) = StateField::Context.set(ctx, &context.name) {
        println!("Switched to {} context", context.name.green());
    } else {
        panic!("failed to switch to {} context", context.name.red());
    }

    Ok(())
}

pub fn ls(ctx: &Ctx) {
    if let Ok(entries) = std::fs::read_dir(ctx.path().join("contexts")) {
        for entry in entries {
            let bytes = entry.unwrap().file_name();
            let context_name = bytes.to_str().unwrap();

            let state = ctx
                .state
                .get(ctx, StateField::Context)
                .unwrap_or(String::from("default"));

            if state == context_name {
                println!("* {}", context_name.green());
            } else {
                println!("  {}", context_name);
            }
        }
    }
}

pub fn rm(ctx: &Ctx, context: String) {
    let context = Context::new(&context);

    if !context.exists(ctx) {
        panic!("context {} does not exist", context.name.red());
    }

    if std::fs::remove_dir_all(context.dir(ctx)).is_ok() {
        println!("Deleted {} context", context.name.green());
    } else {
        panic!("failed to delete {} context", context.name.red());
    }
}

pub fn print(ctx: &Ctx) {
    println!(
        "{}",
        ctx.state
            .get(ctx, StateField::Context)
            .unwrap_or("default".into())
    );
}
