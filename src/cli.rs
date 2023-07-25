use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "quartz")]
#[command(author = "Eduardo R. <contato@edurodrigues.dev>")]
#[command(about = "API Client made into a CLI tool", long_about = None, version)]
pub struct Cli {
    /// Run quartz using a specific handle
    #[arg(short = 'x', value_name = "HANDLE")]
    pub from_handle: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize quartz
    Init { directory: Option<PathBuf> },
    /// Send request using the current handle's endpoint and outputs the response
    Send { handle: Option<String> },
    /// Create a new handle
    Create {
        handle: String,

        /// Set handle's endpoint URL
        #[arg(long)]
        url: Option<String>,

        /// Set handle's endpoint method value
        #[arg(long)]
        method: Option<String>,

        /// Set a query entry in "key=value" format.
        #[arg(long)]
        query: Vec<String>,

        /// Set a header entry in "<key>: <value>" format. This argument can be passed multiple times
        #[arg(long)]
        header: Vec<String>,

        /// Immediatly switches to this handle after creating it.
        #[arg(name = "use", long)]
        switch: bool,
    },
    /// Switch to a handle
    Use { handle: String },
    /// Print the current status of quartz
    Status {
        #[command(subcommand)]
        command: StatusCommands,
    },
    /// Lists available handles
    #[command(alias = "ls")]
    List {
        /// Set a limit for how deep the listing goes in sub-handles
        #[arg(long, value_name = "N")]
        depth: Option<u16>,
    },
    /// Delete the specified handle recursively
    #[command(alias = "rm")]
    Remove {
        /// Endpoint specification
        handle: String,
    },
    /// Print endpoint informations at a handle
    Show { handle: Option<String> },
    /// Open an editor to modify endpoint in use
    Edit {
        #[arg(long)]
        /// Defines the editor to be used for that run, overriding the quartz settings.
        editor: Option<String>,
    },
    /// Manage current handle's endpoint URL
    Url {
        #[command(subcommand)]
        command: EndpointUrlCommands,
    },
    /// Manage current handle's endpoint method
    Method {
        #[command(subcommand)]
        command: EndpointMethodCommands,
    },
    /// Manage current handle's endpoint query
    Query {
        #[command(subcommand)]
        command: EndpointQueryCommands,
    },
    /// Manage current handle's endpoint headers
    Headers {
        /// Add new header entry in "key: value" format
        #[arg(long, value_name = "HEADER")]
        add: Vec<String>,

        /// Remove a header
        #[arg(long, value_name = "KEY")]
        remove: Vec<String>,

        /// Print headers
        #[arg(long)]
        list: bool,
    },
    /// Manage current handle's endpoint request body
    Body {
        /// Expect a new request body via standard input
        #[arg(long)]
        stdin: bool,

        /// Open an editor to modify the endpoint's request body
        #[arg(long, short)]
        edit: bool,

        /// Print request body
        #[arg(long, short)]
        print: bool,
    },
    /// Print request history
    History {
        /// Maximum number of requests to be listed
        #[arg(short = 'n', long, value_name = "N")]
        max_count: Option<usize>,
        /// Format date time output
        #[arg(long, value_name = "FORMAT")]
        date: Option<String>,
    },
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },
    /// Manage current context's variables
    #[command(alias = "var")]
    Variable {
        /// Print a variable value
        #[arg(long, value_name = "KEY")]
        get: Option<String>,

        /// Set a variable: key=value
        #[arg(long, value_name = "VARIABLE")]
        set: Option<String>,

        /// Print all variables
        #[arg(long)]
        list: bool,

        /// Open an editor to modify the context variables file
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
    /// Print the handle for the endpoint in use
    #[command(name = "--endpoint")]
    Endpoint,

    /// Print the context in use
    #[command(name = "--context")]
    Context,
}

#[derive(Debug, Subcommand)]
pub enum EndpointUrlCommands {
    /// Print URL
    #[command(name = "--get")]
    Get,

    /// Set a value for URL
    #[command(name = "--set")]
    Set { url: String },
}

#[derive(Debug, Subcommand)]
pub enum EndpointMethodCommands {
    /// Print method
    #[command(name = "--get")]
    Get,

    /// Set a value for method
    #[command(name = "--set")]
    Set { method: String },
}

#[derive(Debug, Subcommand)]
pub enum EndpointQueryCommands {
    /// Print URL
    #[command(name = "--get")]
    Get { key: Option<String> },

    /// Set a value for URL
    #[command(name = "--set")]
    Set { query: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Open an editor to modify ~/.quartz.toml
    #[command(name = "--edit")]
    Edit,

    /// Print configuration value
    #[command(name = "--get")]
    Get { key: String },

    /// Set a configuration
    #[command(name = "--set")]
    Set { key: String, value: String },

    /// Print ~/.quartz.toml
    #[command(name = "--list")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum ContextCommands {
    /// Create a new context
    Create {
        name: String,
        /// Copy variables from another context
        #[arg(short, long, value_name = "CONTEXT")]
        copy: Option<String>,
    },
    /// Switch to another context
    Use { context: String },
    /// Print all available contexts
    #[command(alias = "ls")]
    List,
    /// Delete a context
    #[command(alias = "rm")]
    Remove { context: String },
}
