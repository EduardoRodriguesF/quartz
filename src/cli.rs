use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "quartz")]
#[command(author = "Eduardo R. <contato@edurodrigues.dev>")]
#[command(about = "Text-based API Client", long_about = None, version)]
pub struct Cli {
    /// Run command with given handle
    #[arg(short = 'x', value_name = "HANDLE")]
    pub from_handle: Option<String>,

    /// Apply environment on endpoint as soon as possible. Allows to get resolved information on
    /// output
    #[arg(short = 'c', long)]
    pub apply_environment: bool,

    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Initialize quartz
    Init { directory: Option<PathBuf> },
    /// Send request using the current handle's endpoint and outputs the response
    Send {
        /// Change a variable when sending the request.
        #[arg(long, short = 'v', value_name = "KEY=VALUE")]
        var: Vec<String>,

        /// Change or include an extra header
        #[arg(long, short = 'H')]
        header: Vec<String>,

        /// Change or include an extra query param
        #[arg(long, short = 'q')]
        query: Vec<String>,

        /// Change request method
        #[arg(long, short = 'X', value_name = "METHOD")]
        request: Option<String>,

        /// Sends data in request body
        #[arg(long, short = 'd')]
        data: Option<String>,

        /// Prevent quartz from following redirects
        #[arg(long)]
        no_follow: bool,

        /// Pass cookie data to request header
        #[arg(long, short = 'b', value_name = "DATA|FILENAME")]
        cookie: Vec<String>,

        /// Which file to write all cookies after a completed request
        #[arg(long, short = 'c', value_name = "FILE")]
        cookie_jar: Option<PathBuf>,
    },
    /// Create a new handle
    Create {
        handle: String,

        /// Set handle's endpoint URL
        #[arg(long)]
        url: Option<String>,

        /// Set handle's endpoint method value
        #[arg(short = 'X', long = "request")]
        method: Option<String>,

        /// Add a key-value pair to the URL query.
        #[arg(short, long, value_name = "KEY=VALUE")]
        query: Vec<String>,

        /// Set a header entry in "<key>: <value>" format. This argument can be passed multiple times
        #[arg(short = 'H', long)]
        header: Vec<String>,

        /// Immediatly switches to this handle after creating it
        #[arg(name = "use", long)]
        switch: bool,
    },
    /// Switch handle or edit its endpoint
    Use {
        handle: Option<String>,

        #[arg(long)]
        url: Option<String>,

        /// HTTP request method
        #[arg(short = 'X', long = "request")]
        method: Option<String>,

        /// Add a parameter the URL query
        #[arg(short, long, value_name = "PARAM")]
        query: Vec<String>,

        /// Add a header entry in "<key>: <value>" format. This argument can be passed multiple times
        #[arg(short = 'H', long)]
        header: Vec<String>,

        /// Make handle empty. Using it with other editing options will write a new endpoint in
        /// place of the old one
        #[arg(long)]
        empty: bool,
    },
    /// Lists available handles
    #[command(name = "ls", alias = "list")]
    Ls {
        /// Set a limit for how deep the listing goes in sub-handles
        #[arg(long, value_name = "N")]
        depth: Option<usize>,
    },
    /// Copy an endpoint from one handle to another
    #[command(name = "cp", alias = "copy")]
    Cp {
        #[arg(long, short = 'r')]
        recursive: bool,

        src: String,
        dest: String,
    },
    /// Move handles
    #[command(name = "mv", alias = "move")]
    Mv { handles: Vec<String> },
    /// Delete handles
    #[command(name = "rm", alias = "remove")]
    Rm {
        /// Delete child handles recursively
        #[arg(long, short = 'r')]
        recursive: bool,

        /// Handles to be removed
        handle: Vec<String>,
    },
    /// Print out endpoint informations
    Show {
        #[command(subcommand)]
        command: ShowCmd,
    },
    /// Open an editor to modify endpoint in use
    Edit {
        #[arg(long)]
        /// Defines the editor to be used for that run, overriding the quartz settings.
        editor: Option<String>,
    },
    /// Manage current endpoint's query params
    Query {
        #[command(subcommand)]
        command: QueryCmd,
    },
    /// Manage current endpoint's headers. Without subcomand, it prints the headers list.
    #[command(alias = "headers")]
    Header {
        #[command(subcommand)]
        command: HeaderCmd,
    },
    /// Manage current handle's endpoint request body
    Body {
        /// Which extension to read body as. E.g.: quartz body --format json edit
        #[arg(long, value_name = "EXT")]
        format: Option<String>,

        #[command(subcommand)]
        command: BodyCmd,
    },
    /// Print information about last request or response
    Last {
        #[command(subcommand)]
        command: Option<LastCmd>,
    },
    /// Print request history
    History {
        /// Maximum number of requests to be listed
        #[arg(short = 'n', long, value_name = "N")]
        max_count: Option<usize>,
    },
    /// Manage project's environments
    #[command(name = "env", alias = "environment")]
    Env {
        #[command(subcommand)]
        command: EnvCmd,
    },
    /// Manage current environment's variables
    #[command(name = "var", alias = "variable")]
    Var {
        #[command(subcommand)]
        command: VarCmd,
    },
    /// Manage configuration for quartz
    Config {
        #[command(subcommand)]
        command: ConfigCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum LastCmd {
    /// Print most recent handle used
    Handle,

    /// Print last request information
    #[command(name = "req", alias = "request")]
    Req,
    /// Print last response information
    #[command(name = "res", alias = "response")]
    Res {
        #[command(subcommand)]
        command: Option<LastResCmd>,
    },
}

#[derive(Debug, Subcommand)]
pub enum LastResCmd {
    Head,
    Body,
}

#[derive(Debug, Subcommand)]
pub enum QueryCmd {
    /// Print query param value
    Get { key: String },

    /// Set query param value
    Set { query: Vec<String> },

    /// Remove query param
    #[command(name = "rm", alias = "remove")]
    Rm { key: Vec<String> },

    /// List all query params
    #[command(name = "ls", alias = "list")]
    Ls,
}

#[derive(Debug, Subcommand)]
pub enum HeaderCmd {
    /// Print a header value
    Get { key: String },

    /// Add new or existent header. Expects "key: value" format
    Set { header: Vec<String> },

    /// Remove a header
    #[command(name = "rm", alias = "remove")]
    Rm { key: Vec<String> },

    /// Print headers
    #[command(name = "ls", alias = "list")]
    Ls,
}

#[derive(Debug, Subcommand)]
pub enum ShowCmd {
    Url,
    Method,
    /// Display endpoint's headers
    Headers {
        key: Option<String>,
    },
    /// Display endpoint's query params
    Query {
        key: Option<String>,
    },
    /// Display endpoint's request body
    Body,
    /// Display current handle
    Handle,
    /// Display current environment
    #[command(name = "env", alias = "environment")]
    Env,

    /// Display environment cookies
    Cookies {
        key: Option<String>,

        /// Filter cookies that match this domain
        #[arg(long, short = 'd')]
        domain: Option<String>,
    },
    /// Generate code snippet for endpoint
    Snippet {
        /// Use a new or overwritten variable
        #[arg(long, short = 'v', value_name = "KEY=VALUE")]
        var: Vec<String>,

        #[command(subcommand)]
        command: SnippetCmd,
    },
    /// Display endpoint configuration file
    Endpoint,
}

#[derive(Debug, Subcommand)]
pub enum SnippetCmd {
    Curl {
        /// Use long form cURL options (--header instead of -H)
        #[arg(long)]
        long: bool,

        /// Split output across multiple lines
        #[arg(long)]
        multiline: bool,
    },
    Http,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Open an editor to modify ~/.quartz.toml
    Edit,

    /// Print configuration value
    Get { key: String },

    /// Set a configuration
    Set { key: String, value: String },

    /// Print ~/.quartz.toml
    #[command(name = "ls", alias = "list")]
    Ls,
}

#[derive(Debug, Subcommand)]
pub enum BodyCmd {
    /// Print request body to stdout
    Show,

    /// Expect a new request body via standard input
    Stdin,

    /// Open an editor to modify the endpoint's request body
    Edit,
}

#[derive(Debug, Subcommand)]
pub enum EnvCmd {
    /// Create a new environment
    Create { name: String },

    /// Switch to another environment
    Use { env: String },

    /// Print all available environments
    #[command(name = "ls", alias = "list")]
    Ls,

    /// Copy variables from a environment to a new or existing one
    #[command(name = "cp", alias = "copy")]
    Cp { src: String, dest: String },

    /// Delete a environment
    #[command(name = "rm", alias = "remove")]
    Rm { env: String },
}

#[derive(Debug, Subcommand)]
pub enum VarCmd {
    /// Open an editor to modify variables
    Edit,

    /// Display variable value
    Get { key: String },

    /// Add a new or existent variable value
    Set { variable: Vec<String> },

    /// Remove variables
    Rm { key: Vec<String> },

    /// Display the list of variables
    #[command(name = "ls", alias = "list")]
    Ls,
}
