use crate::{cli::SnippetCmd as Cmd, snippet, Ctx, PairMap, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd, variables: Vec<String>) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();
    let mut env = ctx.require_env();

    for var in variables {
        env.variables.set(&var);
    }

    endpoint.apply_env(&env);

    match command {
        Cmd::Curl { long, multiline } => {
            let curl = snippet::Curl { long, multiline };

            curl.print(&endpoint)?;
        }
        Cmd::Http => {
            snippet::Http::print(&endpoint)?;
        }
    };

    Ok(())
}
