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
    Endpoint {
        #[command(subcommand)]
        command: EndpointCommands,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum EndpointCommands {
    /// Sends request from endpoint
    Send {
        endpoint: Option<String>,
    },
    Create {
        name: String,

        #[arg(long)]
        url: Option<String>,

        #[arg(long)]
        method: Option<String>,

        /// Header entry in "<key>: <value>" format. This argument can be passed multiple times.
        #[arg(long)]
        header: Vec<String>,
    },
    Use {
        endpoint: String,
    },
    #[command(name = "ls")]
    List,
    Url {
        #[command(subcommand)]
        command: EndpointUrlCommands,
    },
    Method {
        #[command(subcommand)]
        command: EndpointMethodCommands,
    },
    Headers {
        endpoint: Option<String>,

        /// New header entry in "<key>: <value>" format. This argument can be passed multiple times. Overrides duplicates.
        #[arg(long)]
        add: Vec<String>,

        /// Header key to remove from endpoint. This argument can be passed multiple times.
        #[arg(long)]
        remove: Vec<String>,

        /// Prints existing headers.
        #[arg(long)]
        list: bool,
    },
    Body {
        endpoint: Option<String>,

        /// Expects a new request body via standard input.
        #[arg(long)]
        stdin: bool,

        /// Opens an editor to modify the endpoint's request body.
        #[arg(long, short)]
        edit: bool,

        /// Prints request body to standard output.
        #[arg(long, short)]
        print: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum EndpointUrlCommands {
    #[command(name = "--get")]
    Get { endpoint: Option<String> },

    #[command(name = "--set")]
    Set {
        endpoint: Option<String>,
        url: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum EndpointMethodCommands {
    #[command(name = "--get")]
    Get { endpoint: Option<String> },

    #[command(name = "--set")]
    Set {
        endpoint: Option<String>,
        method: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}
