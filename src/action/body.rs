use crate::{cli::BodyCmd as Cmd, validator, Ctx, QuartzResult};
use std::io::Write;

pub struct BodyArgs {
    pub format: Option<String>,
}

pub fn cmd(ctx: &Ctx, command: Cmd, args: BodyArgs) -> QuartzResult {
    match command {
        Cmd::Show => print(ctx),
        Cmd::Stdin => stdin(ctx),
        Cmd::Edit => edit(ctx, args.format)?,
    };

    Ok(())
}

pub fn print(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();

    print!("{}", endpoint.body());
}

pub fn edit(ctx: &Ctx, format: Option<String>) -> QuartzResult {
    let handle = ctx.require_handle();
    let path = handle.dir(ctx).join("body");

    if let Some(format) = format {
        // We cannot validate json for now. If we do so, variable notation will fail because it can
        // generate invalid JSON. For exemple:
        //
        // { "value": {{n}} }
        //
        // n must be a number, so we don't wrap it in quotes. This JSON before variables is
        // invalid. A solution may or may not be done later.
        ctx.edit_with_extension(&path, Some(&format), validator::infallible)?;
    } else {
        ctx.edit(&path, validator::infallible)?;
    }

    Ok(())
}

pub fn stdin(ctx: &Ctx) {
    let handle = ctx.require_handle();

    let mut input = String::new();
    while let Ok(bytes) = std::io::stdin().read_line(&mut input) {
        if bytes == 0 {
            break;
        }
    }

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(handle.dir(ctx).join("body"))
    {
        let _ = file.write_all(input.as_bytes());
    }
}
