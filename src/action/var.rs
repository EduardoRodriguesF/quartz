use crate::{cli::VarCmd as Cmd, context::Variables, Ctx, PairMap, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Edit => edit(ctx)?,
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { variable } => set(ctx, variable)?,
        Cmd::Remove { key } => rm(ctx, key)?,
        Cmd::List => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, key: String) {
    let context = ctx.require_context();
    let v = context
        .variables
        .get(&key)
        .unwrap_or_else(|| panic!("{} variable not set", key));

    println!("{}", v);
}

pub fn set(ctx: &Ctx, variables: Vec<String>) -> QuartzResult {
    let mut context = ctx.require_context();
    for input in variables {
        context.variables.set(&input);
    }

    context.update()?;
    Ok(())
}

pub fn ls(ctx: &Ctx) {
    let context = ctx.require_context();
    println!("{}", context.variables);
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    let context = ctx.require_context();
    ctx.edit(&context.dir().join("variables"), |c| {
        Variables::parse(c);
        Ok(())
    })?;

    Ok(())
}

pub fn rm(ctx: &Ctx, key: String) -> QuartzResult {
    let mut context = ctx.require_context();
    context
        .variables
        .remove(&key)
        .unwrap_or_else(|| panic!("{} variable not set", key));

    context.update()?;
    Ok(())
}
