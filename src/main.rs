mod cli;
mod config;
mod endpoint;
pub mod internals;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use endpoint::Endpoint;
use internals::*;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Create { name } => {
            let endpoint = Endpoint {
                name,
                req: hyper::Request::builder()
                    .uri("https://httpbin.org/get")
                    .method(hyper::Method::GET)
                    .body(())
                    .unwrap(),
            };

            endpoint.write();
        }
        Commands::Layout { command } => match command {
            cli::LayoutCommands::Create { name } => {
                layout::create(&name);
            }
            cli::LayoutCommands::Use { layout } => {
                if !layout::does_exist(&layout) {
                    layout::create(&layout);
                }

                layout::switch(&layout);
            }
            cli::LayoutCommands::Which => {
                println!("{}", layout::which());
            }
            cli::LayoutCommands::List => {
                let which = layout::which();

                for layout in layout::list() {
                    if which == layout {
                        println!("* {}", layout.green());
                    } else {
                        println!("  {}", layout);
                    };
                }
            }
        },
        Commands::Config { command } => match command {
            cli::ConfigCommands::Edit => {
                let _ = std::process::Command::new(config.preferences.editor)
                    .arg(Config::filepath().to_str().unwrap())
                    .status()
                    .expect("Failed to open editor");
            }
        },
    }
}
