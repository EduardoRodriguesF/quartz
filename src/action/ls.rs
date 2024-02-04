use crate::{Ctx, EndpointHandle};
use colored::Colorize;
use std::path::PathBuf;

pub fn cmd(ctx: &Ctx, max_depth: Option<usize>) {
    let max_depth = max_depth.unwrap_or(usize::MAX).max(1);
    let mut current = PathBuf::new();

    if let Some(handle) = EndpointHandle::from_state(ctx) {
        current = handle.dir(ctx)
    }

    // This code is a mess.
    // I'm sorry.
    // It will be refactored sometime.
    struct TraverseEndpoints<'s> {
        f: &'s dyn Fn(&TraverseEndpoints, Vec<EndpointHandle>),
    }
    let traverse_handles = TraverseEndpoints {
        f: &|recurse, handles| {
            for handle in handles {
                let children = handle.children(ctx);

                if !handle.path.is_empty() {
                    let (annotation, method, display_handle) = {
                        let mut ann = " ";
                        let mut m = "---".dimmed();
                        let mut h = handle.handle().normal();

                        if current == handle.dir(ctx) {
                            ann = "*";
                            h = h.green();
                        }

                        if let Some(endpoint) = handle.endpoint(ctx) {
                            m = endpoint.colored_method().bold();
                        }

                        (ann, m, h)
                    };

                    print!("{}  {: >5} {}", annotation, method, display_handle);
                }

                if !children.is_empty() {
                    if handle.path.len() < max_depth {
                        // Avoid extra newline from Specification::QUARTZ usage
                        if !handle.path.is_empty() {
                            println!();
                        }

                        (recurse.f)(recurse, children);
                    } else {
                        println!("{}", " +".dimmed());
                    }
                } else {
                    println!();
                }
            }
        },
    };

    (traverse_handles.f)(&traverse_handles, vec![EndpointHandle::QUARTZ]);
}
