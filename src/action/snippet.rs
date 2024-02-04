use crate::{cli::SnippetCmd as Cmd, snippet, Ctx, PairMap, QuartzResult};

pub fn cmd(ctx: &Ctx, command: Cmd, variables: Vec<String>) -> QuartzResult {
    let (_, mut endpoint) = ctx.require_endpoint();
    let mut context = ctx.require_context();

    for var in variables {
        context.variables.set(&var);
    }

    endpoint.apply_context(&context);

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
