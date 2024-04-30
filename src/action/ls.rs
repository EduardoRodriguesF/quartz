use crate::{Ctx, EndpointHandle};
use colored::{ColoredString, Colorize};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    depth: Option<usize>,
}

pub fn cmd(ctx: &Ctx, args: Args) {
    let max_depth = args.depth.unwrap_or(usize::MAX).max(1);

    let active_handle = if let Some(handle) = EndpointHandle::from_state(ctx) {
        handle.handle()
    } else {
        "".into()
    };

    let mut list: Vec<(ColoredString, String)> = vec![];

    // This code is a mess.
    // I'm sorry.
    // It will be refactored sometime.
    struct TraverseEndpoints<'s> {
        f: &'s dyn Fn(&TraverseEndpoints, Vec<EndpointHandle>, &mut Vec<(ColoredString, String)>),
    }
    let traverse_handles = TraverseEndpoints {
        f: &|recurse, handles, acc| {
            for handle in handles {
                let children = handle.children(ctx);

                if !handle.path.is_empty() {
                    let method = if let Some(endpoint) = handle.endpoint(ctx) {
                        endpoint.colored_method().bold()
                    } else {
                        "---".dimmed()
                    };

                    acc.push((method, handle.handle()));
                }

                if !children.is_empty() {
                    if handle.path.len() < max_depth {
                        // Avoid extra newline from Specification::QUARTZ usage
                        if !handle.path.is_empty() {
                            println!();
                        }

                        (recurse.f)(recurse, children, acc);
                    } else {
                        println!("{}", " +".dimmed());
                    }
                } else {
                    println!();
                }
            }
        },
    };

    // Fills up list
    (traverse_handles.f)(&traverse_handles, vec![EndpointHandle::QUARTZ], &mut list);

    let padding = list.iter().fold(0, |l, (m, _)| l.max(m.len()));

    for (method, handle) in list {
        let (annotation, handle) = if active_handle == handle {
            ("*", handle.green())
        } else {
            (" ", handle.normal())
        };

        println!(
            "{} {:<padding$} {}",
            annotation,
            method,
            handle,
            padding = padding
        );
    }
}
