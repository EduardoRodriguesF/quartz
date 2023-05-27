mod cli;
mod config;
mod endpoint;
pub mod internals;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use endpoint::{Endpoint, EndpointConfig};
use hyper::body::HttpBody;
use internals::*;
use tokio::io::{stdout, AsyncWriteExt as _};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Send { endpoint } => {
            let config = EndpointConfig::from_name(&endpoint);
            let endpoint = Endpoint::from_config(config).unwrap();

            let mut res = endpoint.send().await.unwrap();

            println!("Status: {}", res.status());

            while let Some(chunk) = res.data().await {
                stdout().write_all(&chunk.unwrap()).await.unwrap();
            }
        }
        Commands::Create {
            name,
            url: maybe_url,
            method: maybe_method,
            header,
        } => {
            let mut config = EndpointConfig::new(&name);

            for item in header {
                let splitted_item = item.splitn(2, ": ").collect::<Vec<&str>>();

                if splitted_item.len() <= 1 {
                    panic!("Malformed header argument: {}", item);
                }

                let key = splitted_item[0];
                let value = splitted_item[1];

                config.headers.insert(key.to_string(), value.to_string());
            }

            if let Some(url) = maybe_url {
                config.url = url;
            }

            if let Some(method) = maybe_method {
                config.method = method;
            }

            config.write();
        }
        Commands::Layout { command } => match command {
            cli::LayoutCommands::Create { name } => {
                layout::create(&name);
            }
            cli::LayoutCommands::Use { layout } => {
                if !layout::does_exist(&layout) {
                    layout::create(&layout);
                }

                layout::switch(&layout);
            }
            cli::LayoutCommands::Which => {
                println!("{}", layout::which());
            }
            cli::LayoutCommands::List => {
                let which = layout::which();

                for layout in layout::list() {
                    if which == layout {
                        println!("* {}", layout.green());
                    } else {
                        println!("  {}", layout);
                    };
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
