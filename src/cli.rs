use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "api")] // will be renamed soon
#[command(about = "API Client made into a CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    // Initializes a new layout.
    #[command(arg_required_else_help = true)]
    Init { name: String },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}
