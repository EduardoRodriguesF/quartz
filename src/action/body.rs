use crate::{validator, Ctx, QuartzResult};
use std::io::Write;

pub struct BodyArgs {
    pub stdin: bool,
    pub edit: bool,
    pub print: bool,
}

pub fn cmd(ctx: &Ctx, args: BodyArgs) -> QuartzResult {
    if args.stdin {
        stdin(ctx);
    }

    if args.edit {
        edit(ctx)?;
    }

    if args.print {
        print(ctx);
    }

    Ok(())
}

pub fn print(ctx: &Ctx) {
    let (handle, endpoint) = ctx.require_endpoint();

    print!("{}", endpoint.body(&handle));
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    let handle = ctx.require_handle();
    ctx.edit(&handle.dir(), validator::json)?;

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
        .open(handle.dir().join("body.json"))
    {
        let _ = file.write_all(input.as_bytes());
    }
}
