mod cli;
mod config;
mod endpoint;
pub mod internals;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use endpoint::{Endpoint, EndpointConfig};
use http_body_util::BodyExt;
use internals::*;
use std::collections::HashMap;
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

            while let Some(next) = res.frame().await {
                let frame = next.unwrap();
                if let Some(chunk) = frame.data_ref() {
                    let _ = stdout().write_all(&chunk).await;
                }
            }
        }
        Commands::Create { name } => {
            let mut headers = HashMap::new();
            headers.insert("Content-type".into(), "application/json".into());
            headers.insert("one-more-key".into(), "".into());

            let config = EndpointConfig {
                name,
                method: "GET".to_string(),
                url: "http://httpbin.org/get".to_string(),
                headers,
            };

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
