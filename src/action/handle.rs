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
    pub src: String,
    pub dest: String,
}

pub struct RmArgs {
    pub handle: String,
    pub recursive: bool,
}

pub struct EditArgs {
    pub editor: Option<String>,
}

pub fn create(mut args: CreateArgs) {
    if args.handle.is_empty() {
        panic!("missing endpoint handle");
    }

    let handle = EndpointHandle::from_handle(args.handle);

    if handle.exists() {
        panic!("endpoint already exists");
    }

    let mut endpoint = Endpoint::from(&mut args.config);

    if args.switch {
        if let Ok(()) = StateField::Endpoint.set(&handle.path.join("/")) {
            println!("Switched to {} endpoint", handle.handle().green());
        } else {
            panic!("failed to switch to {} endpoint", handle.handle().red());
        }
    }

    handle.write();
    endpoint.write(&handle);
}

pub fn switch(ctx: &Ctx, mut args: SwitchArgs) {
    let handle = if let Some(handle) = args.handle {
        let handle = ctx.require_input_handle(&handle);

        if !handle.dir().exists() {
            panic!("endpoint does not exist");
        }

        if StateField::Endpoint.set(&handle.path.join("/")).is_ok() {
            println!("Switched to {} endpoint", handle.handle().green());
        } else {
            panic!("failed to switch to {} endpoint", handle.handle().red());
        }

        handle
    } else {
        ctx.require_handle()
    };

    if args.empty {
        handle.make_empty();
    }

    if !args.config.has_changes() {
        return;
    }

    let mut endpoint = handle.endpoint().unwrap_or(Endpoint::default());

    endpoint.update(&mut args.config);
    endpoint.write(&handle);
}

pub fn cp(ctx: &Ctx, args: CpArgs) {
    let src = ctx.require_input_handle(&args.src);
    let dest = EndpointHandle::from_handle(args.dest);

    let mut endpoint = src
        .endpoint()
        .unwrap_or_else(|| panic!("no endpoint at {}", src.handle().red()));

    if !dest.exists() {
        dest.write();
    }

    endpoint.write(&dest);
}

pub fn rm(ctx: &Ctx, args: RmArgs) {
    let handle = ctx.require_input_handle(&args.handle);

    if handle.children().len() > 0 && !args.recursive {
        panic!(
            "{} has child handles. Use {} option to confirm",
            handle.handle(),
            "-r".red()
        )
    }

    if std::fs::remove_dir_all(handle.dir()).is_ok() {
        println!("Deleted endpoint {}", handle.handle());
    } else {
        panic!("failed to delete endpoint {}", handle.handle());
    }
}

pub fn edit(ctx: &mut Ctx, args: EditArgs) -> QuartzResult {
    let handle = ctx.require_handle();

    if let Some(editor) = args.editor {
        ctx.config.preferences.editor = editor;
    }

    ctx.edit(
        &handle.dir().join("endpoint.toml"),
        validator::toml_as::<Endpoint>,
    )?;

    Ok(())
}
