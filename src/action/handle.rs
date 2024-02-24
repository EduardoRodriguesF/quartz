use std::{collections::VecDeque, process::exit};

use crate::{
    endpoint::{Endpoint, EndpointHandle, EndpointInput},
    validator, Ctx, QuartzResult, StateField,
};
use colored::Colorize;

#[derive(Default)]
pub struct CreateArgs {
    pub handle: String,
    pub config: EndpointInput,
    pub switch: bool,
}

#[derive(Default)]
pub struct SwitchArgs {
    pub handle: Option<String>,
    pub config: EndpointInput,
    pub empty: bool,
}

pub struct CpArgs {
    pub recursive: bool,
    pub src: String,
    pub dest: String,
}

pub struct RmArgs {
    pub handles: Vec<String>,
    pub recursive: bool,
}

pub struct EditArgs {
    pub editor: Option<String>,
}

pub fn create(ctx: &Ctx, mut args: CreateArgs) {
    if args.handle.is_empty() {
        panic!("missing endpoint handle");
    }

    let handle = EndpointHandle::from(args.handle);

    if handle.exists(ctx) {
        panic!("endpoint already exists");
    }

    let mut endpoint = Endpoint::from(&mut args.config);
    endpoint.set_handle(ctx, &handle);

    if args.switch {
        if let Ok(()) = StateField::Endpoint.set(ctx, &handle.path.join("/")) {
            println!("Switched to {} endpoint", handle.handle().green());
        } else {
            panic!("failed to switch to {} endpoint", handle.handle().red());
        }
    }

    handle.write(ctx);
    endpoint.write();
}

pub fn switch(ctx: &Ctx, mut args: SwitchArgs) {
    let handle = if let Some(handle) = args.handle {
        let handle = ctx.require_input_handle(&handle);

        if !handle.dir(ctx).exists() {
            panic!("endpoint does not exist");
        }

        if StateField::Endpoint
            .set(ctx, &handle.path.join("/"))
            .is_ok()
        {
            println!("Switched to {} endpoint", handle.handle().green());
        } else {
            panic!("failed to switch to {} endpoint", handle.handle().red());
        }

        handle
    } else {
        ctx.require_handle()
    };

    if args.empty {
        handle.make_empty(ctx);
    }

    if !args.config.has_changes() {
        return;
    }

    let mut endpoint = handle
        .endpoint(ctx)
        .unwrap_or(Endpoint::new(handle.dir(ctx)));

    endpoint.update(&mut args.config);
    endpoint.write();
}

pub fn cp(ctx: &Ctx, args: CpArgs) {
    let src = ctx.require_input_handle(&args.src);
    if !src.exists(ctx) {
        panic!("no such handle: {}", src.handle());
    }

    let mut queue = VecDeque::<EndpointHandle>::new();
    queue.push_back(src);

    while let Some(mut src) = queue.pop_front() {
        let endpoint = src.endpoint(ctx);

        if args.recursive {
            for child in src.children(ctx) {
                queue.push_back(child);
            }
        }

        src.replace(&args.src, &args.dest);
        src.write(ctx);

        if let Some(mut endpoint) = endpoint {
            endpoint.set_handle(ctx, &src);
            endpoint.write();
        }
    }
}

pub fn rm(ctx: &Ctx, args: RmArgs) {
    let mut exit_code = 0;

    for name in args.handles {
        let handle = EndpointHandle::from(&name);

        if !handle.exists(ctx) {
            exit_code = 1;
            eprintln!("no such handle: {name}");
            continue;
        }

        if handle.children(ctx).len() > 0 && !args.recursive {
            exit_code = 1;
            eprintln!(
                "{} has child handles. Use -r option to confirm",
                handle.handle(),
            );
            continue;
        }

        if std::fs::remove_dir_all(handle.dir(ctx)).is_ok() {
            println!("Deleted endpoint {}", handle.handle());
        } else {
            exit_code = 1;
            eprintln!("failed to delete endpoint {}", handle.handle());
        }
    }

    exit(exit_code);
}

pub fn edit(ctx: &mut Ctx, args: EditArgs) -> QuartzResult {
    let handle = ctx.require_handle();

    if let Some(editor) = args.editor {
        ctx.config.preferences.editor = editor;
    }

    ctx.edit(
        &handle.dir(ctx).join("endpoint.toml"),
        validator::toml_as::<Endpoint>,
    )?;

    Ok(())
}
