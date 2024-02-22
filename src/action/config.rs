use crate::{cli::ConfigCmd as Cmd, config::ConfigBuilder, validator, Config, Ctx, QuartzResult};

pub fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Edit => edit(ctx)?,
        Cmd::Set { key, value } => set(ctx, key, value),
        Cmd::Ls => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, key: String) {
    let value: String = match key.as_str() {
        "preferences.editor" => ctx.config.preferences.editor.clone(),
        "preferences.pager" => ctx.config.preferences.pager.clone(),
        "ui.colors" => ctx.config.ui.colors.to_string(),
        _ => panic!("invalid key"),
    };

    println!("{value}");
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    ctx.edit(&Config::filepath(), validator::toml_as::<ConfigBuilder>)?;

    Ok(())
}

pub fn set(ctx: &mut Ctx, key: String, value: String) {
    match key.as_str() {
        "preferences.editor" => ctx.config.preferences.editor = value,
        "preferences.pager" => ctx.config.preferences.pager = value,
        "ui.colors" => ctx.config.ui.colors = matches!(value.as_str(), "true"),
        _ => panic!("invalid key"),
    };

    if ctx.config.write().is_err() {
        panic!("failed to save config change");
    }
}

pub fn ls(ctx: &Ctx) {
    let content = toml::to_string(&ctx.config)
        .unwrap_or_else(|_| panic!("could not parse configuration file"));

    println!("{content}");
}
