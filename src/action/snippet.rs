use crate::{cli::SnippetCmd as Cmd, endpoint::EndpointPatch, snippet, Ctx, PairMap, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Use a new or overwritten variable
    #[arg(long = "var", short = 'v', value_name = "KEY=VALUE")]
    variables: Vec<String>,

    #[command(flatten)]
    patch: EndpointPatch,

    #[command(subcommand)]
    command: crate::cli::SnippetCmd,
}

pub fn cmd(ctx: &Ctx, mut args: Args) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();
    let mut env = ctx.require_env();

    for var in args.variables {
        env.variables.set(&var);
    }

    endpoint.update(&mut args.patch);
    endpoint.apply_env(&env);

    match args.command {
        Cmd::Curl(curl) => curl.print(&mut endpoint)?,
        Cmd::Http => snippet::Http::print(&mut endpoint)?,
    };

    Ok(())
}
