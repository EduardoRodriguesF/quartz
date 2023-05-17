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
    Layout {
        #[command(subcommand)]
        command: LayoutCommands,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum LayoutCommands {
    /// Creates a new layout.
    Create { name: String },

    /// Lists existing layouts.
    #[command(name = "ls")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}
