mod cli;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Init { name } => {
            std::fs::create_dir_all(format!("./.api-prototype/{}", name)).expect("Could not create");
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
