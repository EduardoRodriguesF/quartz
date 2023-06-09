use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "quartz")]
#[command(author = "Eduardo R. <contato@edurodrigues.dev>")]
#[command(about = "API Client made into a CLI tool", long_about = None, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        directory: Option<PathBuf>,
    },
    /// Sends request from endpoint
    Send,
    /// Creates a new endpoint
    Create {
        /// New endpoint specification
        specs: Vec<String>,

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
    Use {
        /// Endpoint specification
        endpoint: Vec<String>,
    },
    Status {
        #[command(subcommand)]
        command: StatusCommands,
    },
    /// Lists available endpoints
    #[command(alias = "ls")]
    List {
        /// Set a limit for printing nested endopoints
        #[arg(long)]
        depth: Option<u16>,
    },
    /// Delete endpoint
    #[command(alias = "rm")]
    Remove {
        /// Endpoint specification
        endpoint: Vec<String>,
    },
    /// Print endpoint configuration
    Show,
    /// Opens an editor to modify endpoint configuration
    Edit {
        #[arg(long)]
        editor: Option<String>,
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
    History {
        /// Maximum number of requests to be listed
        #[arg(short = 'n', long)]
        max_count: Option<usize>,
        /// Format date time output
        #[arg(long)]
        date: Option<String>,
    },
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },
    #[command(alias = "var")]
    Variable {
        #[arg(long)]
        get: Option<String>,

        #[arg(long)]
        set: Option<String>,

        #[arg(long)]
        list: bool,

        #[arg(short, long)]
        edit: bool,
    },
    /// Manage configuration for quartz
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum StatusCommands {
    #[command(name = "--endpoint")]
    Endpoint,

    #[command(name = "--context")]
    Context,
}

#[derive(Debug, Subcommand)]
pub enum EndpointUrlCommands {
    /// Get URL value
    #[command(name = "--get")]
    Get,

    /// Set URL value
    #[command(name = "--set")]
    Set { url: String },
}

#[derive(Debug, Subcommand)]
pub enum EndpointMethodCommands {
    /// Get method value
    #[command(name = "--get")]
    Get,

    /// Set method value
    #[command(name = "--set")]
    Set {
        /// New method
        method: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    #[command(name = "--edit")]
    Edit,
}

#[derive(Debug, Subcommand)]
pub enum ContextCommands {
    /// Create a new context
    Create {
        /// The new context's name
        name: String,
        /// Specify another context to copy properties from
        #[arg(short, long)]
        copy: Option<String>,
    },
    Use {
        context: String,
    },
    #[command(alias = "ls")]
    List,
    #[command(alias = "rm")]
    Remove {
        context: String,
    },
}
