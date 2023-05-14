mod cli;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Init { name } => todo!(),
        Commands::Config { command } => match command {
            cli::ConfigCommands::Edit => {
                let status = std::process::Command::new(config.preferences.editor)
                    .arg(Config::filepath().to_str().unwrap())
                    .status()
                    .expect("Failed to open editor");
            }
        },
    }
}
