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
                create_layout(&name);
            }
            cli::LayoutCommands::Use { layout } => {
                if !does_layout_exists(&layout) {
                    create_layout(&layout);
                }

                switch_layout(&layout);
            }
            cli::LayoutCommands::List => {
                for layout in layout_list() {
                    println!("{}", layout);
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

fn create_layout(name: &str) {
    std::fs::create_dir_all(format!("./.api-prototype/{}", name))
        .expect(&format!("Could not create layout: {}", name));
}

fn layout_list() -> Vec<String> {
    if let Ok(files) = std::fs::read_dir("./.api-prototype") {
        return files
            .map(|f| f.unwrap().file_name().to_str().unwrap().to_string())
            .collect();
    };

    Vec::<String>::new()
}

fn does_layout_exists(layout: &String) -> bool {
    layout_list().contains(layout)
}

fn switch_layout(layout: &String) {
    println!("switched to {}", layout);
}
