use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "quartz")]
#[command(about = "API Client made into a CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Sends request from endpoint
    Send {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,
    },
    /// Creates a new endpoint
    Create {
        /// Friendly name for the endpoint
        name: String,

        /// Set URL value
        #[arg(long)]
        url: Option<String>,

        /// Set method value
        #[arg(long)]
        method: Option<String>,

        /// Set header entry in "<key>: <value>" format. This argument can be passed multiple times
        #[arg(long)]
        header: Vec<String>,

        /// Switches to the new endpoint
        #[arg(name = "use", long)]
        switch: bool,
    },
    /// Switch to a given endpoint
    Use { endpoint: String },
    /// Lists available endpoints
    #[command(alias = "ls")]
    List,
    /// Delete endpoint(s)
    #[command(alias = "rm")]
    Remove { endpoints: Vec<String> },
    /// Rename endpoint
    Rename {
        endpoint: String,
        new_name: String,
    },
    /// Manage endpoint url and its params
    Url {
        #[command(subcommand)]
        command: EndpointUrlCommands,
    },
    /// Manage endpoint method
    Method {
        #[command(subcommand)]
        command: EndpointMethodCommands,
    },
    /// Manage endpoint headers
    Headers {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,

        /// New header entry in "<key>: <value>" format. This argument can be passed multiple times. Overrides duplicates
        #[arg(long)]
        add: Vec<String>,

        /// Header key to remove from endpoint. This argument can be passed multiple times
        #[arg(long)]
        remove: Vec<String>,

        /// Print existing headers
        #[arg(long)]
        list: bool,
    },
    /// Manage endpoint request body
    Body {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,

        /// Expect a new request body via standard input
        #[arg(long)]
        stdin: bool,

        /// Opens an editor to modify the endpoint's request body
        #[arg(long, short)]
        edit: bool,

        /// Print request body to standard output
        #[arg(long, short)]
        print: bool,
    },
    /// Manage configuration for quartz
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum EndpointUrlCommands {
    /// Get URL value
    #[command(name = "--get")]
    Get {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,
    },

    /// Set URL value
    #[command(name = "--set")]
    Set {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,

        url: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum EndpointMethodCommands {
    /// Get method value
    #[command(name = "--get")]
    Get {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,
    },

    /// Set method value
    #[command(name = "--set")]
    Set {
        /// Execute command in specified endpoint
        #[arg(long)]
        endpoint: Option<String>,

        /// New method
        method: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}
