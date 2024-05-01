use std::collections::VecDeque;
use std::process::ExitCode;

use crate::{
    endpoint::{Endpoint, EndpointHandle, EndpointPatch},
    validator, Ctx, QuartzResult, StateField,
};
use colored::Colorize;

#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    handle: String,

    #[command(flatten)]
    patch: EndpointPatch,

    /// Immediatly switches to this handle after creating it
    #[arg(name = "use", long)]
    switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    handle: Option<String>,

    #[command(flatten)]
    patch: EndpointPatch,

    /// Make handle empty. Using it with other editing options will write a new endpoint in
    /// place of the old one
    #[arg(long)]
    empty: bool,
}

#[derive(clap::Args, Debug)]
pub struct CpArgs {
    #[arg(long, short = 'r')]
    recursive: bool,

    src: String,
    dest: String,
}

#[derive(clap::Args, Debug)]
pub struct MvArgs {
    handles: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    /// Delete child handles recursively
    #[arg(long, short = 'r')]
    recursive: bool,

    /// Handles to be removed
    #[arg(name = "HANDLE", required = true)]
    handles: Vec<String>,
}

pub fn create(ctx: &Ctx, mut args: CreateArgs) {
    if args.handle.is_empty() {
        panic!("missing endpoint handle");
    }

    let handle = EndpointHandle::from(args.handle);

    if handle.exists(ctx) {
        panic!("endpoint already exists");
    }

    let mut endpoint = Endpoint::from(&mut args.patch);
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

pub fn switch(ctx: &mut Ctx, mut args: SwitchArgs) {
    let handle = if let Some(handle) = args.handle {
        let handle = EndpointHandle::from(handle);

        if !handle.exists(ctx) {
            eprint!("Handle {} doesn't exist", handle.handle().red(),);

            if ctx.confirm("Do you wish to create it?") {
                return create(
                    ctx,
                    CreateArgs {
                        handle: handle.handle(),
                        patch: args.patch,
                        switch: true,
                    },
                );
            } else {
                ctx.code(ExitCode::FAILURE);
                return;
            }
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

    if !args.patch.has_changes() {
        return;
    }

    let mut endpoint = handle
        .endpoint(ctx)
        .unwrap_or(Endpoint::new(handle.dir(ctx)));

    endpoint.update(&mut args.patch);
    endpoint.write();
}

pub fn cp(ctx: &Ctx, args: CpArgs) -> QuartzResult {
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
                queue.push_back(child.clone());
            }
        }

        src.replace(&args.src, &args.dest);
        src.write(ctx);

        if let Some(mut endpoint) = endpoint {
            endpoint.set_handle(ctx, &src);
            endpoint.write();
        }
    }

    Ok(())
}

pub fn rm(ctx: &mut Ctx, args: RmArgs) -> QuartzResult {
    for name in args.handles {
        let handle = EndpointHandle::from(&name);

        if !handle.exists(ctx) {
            ctx.code(ExitCode::FAILURE);
            eprintln!("no such handle: {name}");
            continue;
        }

        if !handle.children(ctx).is_empty() && !args.recursive {
            ctx.code(ExitCode::FAILURE);
            eprintln!(
                "{} has child handles. Use -r option to confirm",
                handle.handle(),
            );
            continue;
        }

        if std::fs::remove_dir_all(handle.dir(ctx)).is_ok() {
            println!("Deleted endpoint {}", handle.handle());
        } else {
            ctx.code(ExitCode::FAILURE);
            eprintln!("failed to delete endpoint {}", handle.handle());
        }
    }

    Ok(())
}

pub fn mv(ctx: &mut Ctx, mut args: MvArgs) -> QuartzResult {
    if args.handles.is_empty() {
        panic!("no handles specified");
    }

    if args.handles.len() == 1 {
        panic!("missing target handle");
    }

    let dest = EndpointHandle::from(args.handles.pop().unwrap());
    let mut original_handles = Vec::<EndpointHandle>::new();
    let mut queue = VecDeque::<(&str, EndpointHandle)>::new();

    for arg in &args.handles {
        let handle = EndpointHandle::from(arg);
        if !handle.exists(ctx) {
            ctx.code(ExitCode::FAILURE);
            eprintln!("no such handle: {arg}");
            continue;
        }

        original_handles.push(handle);
        queue.push_back((arg, EndpointHandle::from(arg)));
    }

    while let Some((src, mut handle)) = queue.pop_front() {
        let mut dest = EndpointHandle::from(dest.handle()); // copy
        for child in handle.children(ctx) {
            queue.push_back((src, child));
        }

        let maybe_endpoint = handle.endpoint(ctx);

        if args.handles.len() >= 2 {
            dest.path.push(handle.path.last().unwrap().to_string());
        }

        handle.replace(src, &dest.handle());
        handle.write(ctx);

        if let Some(mut endpoint) = maybe_endpoint {
            endpoint.set_handle(ctx, &handle);
            endpoint.write();
        }
    }

    for handle in original_handles {
        let _ = std::fs::remove_dir_all(handle.dir(ctx));
    }

    Ok(())
}

pub fn edit(ctx: &mut Ctx) -> QuartzResult {
    let handle = ctx.require_handle();

    ctx.edit(
        &handle.dir(ctx).join("endpoint.toml"),
        validator::toml_as::<Endpoint>,
    )?;

    Ok(())
}
