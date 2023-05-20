mod cli;
mod config;
mod endpoint;
pub mod internals;

use std::collections::HashMap;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use endpoint::{Endpoint, EndpointConfig};
use internals::*;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Create { name } => {
            let mut headers = HashMap::new();
            headers.insert("Content-type".into(), "application/json".into());
            headers.insert("one-more-key".into(), "".into());

            let config = EndpointConfig {
                name,
                method: "GET".to_string(),
                url: "https://httpbin.org/get".to_string(),
                headers,
            };

            config.write();
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
