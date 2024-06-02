use std::vec::Vec;

use crate::{endpoint, Ctx, EndpointHandle};
use colored::Colorize;

#[derive(Default)]
enum UsageState {
    #[default]
    NotUsing,
    Using,
    UsingHiddenChild,
}

struct Output {
    method: Option<String>,
    handle: String,
    has_more: bool,
    usage: UsageState,
}

impl Output {
    fn builder() -> OutputBuilder {
        OutputBuilder::default()
    }
}

#[derive(Default)]
struct OutputBuilder {
    method: Option<String>,
    handle: Option<String>,
    has_more: bool,
    usage: UsageState,
}

impl OutputBuilder {
    fn method(&mut self, method: String) -> &mut Self {
        self.method = Some(method);
        self
    }

    fn handle(&mut self, handle: String) -> &mut Self {
        self.handle = Some(handle);
        self
    }

    fn has_more(&mut self, has_more: bool) -> &mut Self {
        self.has_more = has_more;
        self
    }

    fn usage(&mut self, usage: UsageState) -> &mut Self {
        self.usage = usage;
        self
    }

    fn build(self) -> Result<Output, ()> {
        let handle = self.handle.ok_or(())?;

        if handle.is_empty() {
            return Err(());
        }

        Ok(Output {
            method: self.method,
            handle,
            has_more: self.has_more,
            usage: self.usage,
        })
    }
}

fn print_outputs(list: Vec<Output>) {
    let padding = list
        .iter()
        .map(|e| e.method.as_ref().unwrap_or(&"---".to_string()).len())
        .fold(0, |l, e| l.max(e));

    for output in list {
        let (usage_mark, name) = match output.usage {
            UsageState::NotUsing => (" ", output.handle.normal()),
            UsageState::Using => ("*", output.handle.green()),
            UsageState::UsingHiddenChild => ("*", output.handle.normal()),
        };

        let more_info = if output.has_more {
            " +".dimmed()
        } else {
            "".normal()
        };

        let method = if let Some(method) = output.method {
            endpoint::colored_method(&method).bold()
        } else {
            "---".dimmed()
        };

        println!(
            "{} {:<padding$} {}{}",
            usage_mark,
            method,
            name,
            more_info,
            padding = padding
        );
    }
}

#[derive(clap::Args, Debug)]
pub struct Args {
    /// See the children of a specific handle
    handle: Option<String>,

    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    depth: Option<usize>,
}

pub fn cmd(ctx: &Ctx, args: Args) {
    let max_depth = args.depth.unwrap_or(usize::MAX).max(1);
    let active_handle = EndpointHandle::from_state(ctx);
    let mut output_list: Vec<Output> = vec![];

    let tree = if let Some(name) = args.handle {
        let handle = EndpointHandle::from(&name);

        if !handle.exists(ctx) {
            panic!("no such handle: {name}");
        }

        handle.tree(ctx)
    } else {
        EndpointHandle::QUARTZ.tree(ctx)
    };

    let mut queue = vec![&tree.root];

    while let Some(node) = queue.pop() {
        let mut builder = Output::builder();

        builder.handle(node.value.handle());

        if let Some(endpoint) = node.value.endpoint(ctx) {
            builder.method(endpoint.method);
        } else {
            builder.method("---".into());
        }

        if let Some(active_handle) = &active_handle {
            if node.value.handle() == active_handle.handle() {
                builder.usage(UsageState::Using);
            }
        }

        if node.value.depth() > max_depth {
            if node.children.is_empty() {
                builder.has_more(true);

                if let Some(active_handle) = &active_handle {
                    if active_handle.handle().starts_with(&node.value.handle()) {
                        builder.usage(UsageState::UsingHiddenChild);
                    }
                }
            }

            continue;
        }

        if let Ok(output) = builder.build() {
            output_list.push(output);
        }

        for child in node.children.iter() {
            queue.push(&child);
        }
    }

    print_outputs(output_list);
}
