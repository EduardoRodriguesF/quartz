mod cli;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;

fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Layout { command } => match command {
            cli::LayoutCommands::Create { name } => {
                std::fs::create_dir_all(format!("./.api-prototype/{}", name))
                    .expect(&format!("Could not create layout: {}", name));
            }
            cli::LayoutCommands::List => {
                match std::fs::read_dir("./.api-prototype") {
                    Ok(files) => {
                        for file in files {
                            println!("{}", file.unwrap().file_name().to_str().unwrap());
                        }
                    }
                    Err(_) => (),
                };
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
