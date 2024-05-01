use crate::{cli::ConfigCmd as Cmd, validator, Config, Ctx, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    key: String,
    value: String,
}

pub fn cmd(ctx: &mut Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => get(ctx, args),
        Cmd::Edit => edit(ctx)?,
        Cmd::Set(args) => set(ctx, args),
        Cmd::Ls => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: &Ctx, args: GetArgs) {
    let value = match args.key.as_str() {
        "preferences.editor" => ctx.config.preferences.editor(),
        "preferences.pager" => ctx.config.preferences.pager(),
        "ui.colors" => ctx.config.ui.colors().to_string(),
        _ => panic!("invalid key"),
    };

    println!("{value}");
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    ctx.edit(&Config::filepath(), validator::toml_as::<Config>)?;

    Ok(())
}

pub fn set(ctx: &mut Ctx, args: SetArgs) {
    match args.key.as_str() {
        "preferences.editor" => ctx.config.preferences.set_editor(args.value),
        "preferences.pager" => ctx.config.preferences.set_pager(args.value),
        "ui.colors" => ctx
            .config
            .ui
            .set_colors(matches!(args.value.as_str(), "true")),
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
