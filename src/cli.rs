use crate::action;
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
    Init(action::init::Args),
    /// Send request using the current handle's endpoint and outputs the response
    Send(action::send::Args),
    /// Create a new handle
    Create(action::handle::CreateArgs),
    /// Switch handle or edit its endpoint
    Use(action::handle::SwitchArgs),

    /// Lists available handles
    #[command(name = "ls", alias = "list")]
    Ls(action::ls::Args),

    /// Copy an endpoint from one handle to another
    #[command(name = "cp", alias = "copy")]
    Cp(action::handle::CpArgs),

    /// Move handles
    #[command(name = "mv", alias = "move")]
    Mv(action::handle::MvArgs),

    /// Delete handles
    #[command(name = "rm", alias = "remove")]
    Rm(action::handle::RmArgs),

    /// Print out endpoint informations
    Show {
        #[command(subcommand)]
        command: ShowCmd,
    },

    /// Open an editor to modify endpoint in use
    Edit,

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
    Body(action::body::Args),
    /// Print information about last request or response
    Last {
        #[command(subcommand)]
        command: Option<LastCmd>,
    },
    /// Print request history
    History(action::history::Args),
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
    Get(action::query::GetArgs),

    /// Set query param value
    Set(action::query::SetArgs),

    /// Remove query param
    #[command(name = "rm", alias = "remove")]
    Rm(action::query::RmArgs),

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
    Cookies(action::cookie::PrintArgs),
    /// Generate code snippet for endpoint
    Snippet(action::snippet::Args),
    /// Display endpoint configuration file
    Endpoint,
}

#[derive(Debug, Subcommand)]
pub enum SnippetCmd {
    Curl(crate::snippet::Curl),
    Http,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Open an editor to modify ~/.quartz.toml
    Edit,

    /// Print configuration value
    Get(action::config::GetArgs),

    /// Set a configuration
    Set(action::config::SetArgs),

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
    Create(action::env::CreateArgs),

    /// Switch to another environment
    Use(action::env::SwitchArgs),

    /// Print all available environments
    #[command(name = "ls", alias = "list")]
    Ls,

    /// Copy variables from a environment to a new or existing one
    #[command(name = "cp", alias = "copy")]
    Cp(action::env::CpArgs),

    /// Delete a environment
    #[command(name = "rm", alias = "remove")]
    Rm(action::env::RmArgs),
    Header {
        #[command(subcommand)]
        command: HeaderEnvCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum HeaderEnvCmd {
    Set { headers: Vec<String> },
    Ls,
    Rm(action::env::HeaderRmArgs),
}

#[derive(Debug, Subcommand)]
pub enum VarCmd {
    /// Open an editor to modify variables
    Edit,

    /// Display variable value
    Get(action::var::GetArgs),

    /// Add a new or existent variable value
    Set(action::var::SetArgs),

    /// Remove variables
    Rm(action::var::RmArgs),

    /// Display the list of variables
    #[command(name = "ls", alias = "list")]
    Ls,
}
