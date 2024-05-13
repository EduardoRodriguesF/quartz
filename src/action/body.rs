use crate::{cli::BodyCmd as Cmd, validator, Ctx, QuartzResult};
use std::io::Write;

const POSSIBLE_EXT: [&str; 3] = ["json", "html", "xml"];

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Which extension to read body as. E.g.: quartz body --format json edit
    #[arg(long, value_name = "EXT")]
    format: Option<String>,

    #[command(subcommand)]
    command: crate::cli::BodyCmd,
}

pub fn cmd(ctx: &Ctx, args: Args) -> QuartzResult {
    match args.command {
        Cmd::Show => print(ctx),
        Cmd::Stdin => stdin(ctx),
        Cmd::Edit => edit(ctx, args.format)?,
    };

    Ok(())
}

pub fn print(ctx: &Ctx) {
    let (_, mut endpoint) = ctx.require_endpoint();

    if let Some(body) = endpoint.body() {
        print!("{body}");
    }
}

pub fn edit(ctx: &Ctx, format: Option<String>) -> QuartzResult {
    let handle = ctx.require_handle();
    let path = handle.dir(ctx).join("body");

    let format = if format.is_some() {
        format
    } else {
        let endpoint = ctx.require_endpoint_from_handle(&handle);

        if let Some(content) = endpoint.headers.get("content-type") {
            let ext = POSSIBLE_EXT.iter().find_map(|ext| {
                if content.contains(*ext) {
                    Some(ext.to_string())
                } else {
                    None
                }
            });

            ext
        } else {
            None
        }
    };

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
