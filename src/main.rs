mod cli;
mod config;
pub mod internals;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use internals::*;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Layout { command } => match command {
            cli::LayoutCommands::Create { name } => {
                layout::create(&name);
            }
            cli::LayoutCommands::Use { layout } => {
                if !layout::does_exist(&layout) {
                    layout::create(&layout);
                }

                layout::switch(&layout);
            },
            cli::LayoutCommands::Which => {
                println!("{}", layout::which());
            },
            cli::LayoutCommands::List => {
                let which = layout::which();

                for layout in layout::list() {
                    let mark = if which == layout { "*" } else { " " };

                    println!("{} {}", mark, layout);
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
