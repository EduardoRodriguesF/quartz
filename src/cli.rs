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
    #[command(arg_required_else_help = true)]
    New {
        #[command(subcommand)]
        command: NewCommands,
    },
    #[command(name = "ls")]
    List,
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum NewCommands {
    /// Creates a new layout.
    #[command(name = "-l")]
    Layout { name: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}
