mod cli;
mod config;
mod endpoint;

use core::panic;
use std::{path::Path, io::Write, thread::panicking, process::exit};

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use endpoint::Endpoint;
use hyper::{body::HttpBody, Body, Client};
use tokio::io::{stdout, AsyncWriteExt as _};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let config = Config::parse();

    match args.command {
        Commands::Endpoint { command } => match command {
            cli::EndpointCommands::Send { endpoint } => {
                let endpoint = Endpoint::from_name(&endpoint);
                let req = endpoint.as_request().expect("Malformed request.");
                let client = Client::new();

                let mut res = client.request(req).await.unwrap();

                println!("Status: {}", res.status());

                while let Some(chunk) = res.data().await {
                    stdout().write_all(&chunk.unwrap()).await.unwrap();
                }
            }
            cli::EndpointCommands::Create {
                name,
                url: maybe_url,
                method: maybe_method,
                header,
            } => {
                let mut config = Endpoint::new(&name);

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
            cli::EndpointCommands::Use { endpoint } => {
                if !Path::new(".quartz").join("endpoints").join(&endpoint).is_dir() {
                    eprintln!("Endpoint {} does not exist", &endpoint.red());
                    exit(1);
                }

                let state_file = std::fs::OpenOptions::new()
                    .truncate(true)
                    .create(true)
                    .write(true)
                    .open(Path::new(".quartz").join("state"));

                if let Ok(()) = state_file.unwrap().write_all(endpoint.as_bytes()) {
                    println!("Switched to {} endpoint", endpoint.green());
                } else {
                    panic!("Failed to switch to {} endpoint", endpoint.red());
                }
            }
            cli::EndpointCommands::List => {
                let mut current = String::new();

                if let Some(endpoint) = Endpoint::try_from_state() {
                    current = endpoint.name
                }

                if let Ok(files) = std::fs::read_dir(Path::new(".quartz").join("endpoints")) {
                    for maybe_file in files {
                        if let Ok(file) = maybe_file {
                            if let Some(file_name) = file.file_name().to_str() {
                                if current == file_name {
                                    println!("* {}", file_name.green());
                                } else {
                                    println!("  {}", file_name);
                                }
                            }
                        }
                    }
                }
            }
            cli::EndpointCommands::Url { command } => match command {
                cli::EndpointUrlCommands::Get { endpoint } => {
                    let endpoint = Endpoint::from_name(&endpoint);

                    println!("{}", endpoint.url);
                }
                cli::EndpointUrlCommands::Set { endpoint, url } => {
                    let mut endpoint = Endpoint::from_name(&endpoint);

                    endpoint.url = url;

                    endpoint.update();
                }
            },
            cli::EndpointCommands::Method { command } => match command {
                cli::EndpointMethodCommands::Get { endpoint } => {
                    let endpoint = Endpoint::from_name(&endpoint);

                    println!("{}", endpoint.method);
                }
                cli::EndpointMethodCommands::Set { endpoint, method } => {
                    let mut endpoint = Endpoint::from_name(&endpoint);

                    endpoint.method = method.to_uppercase();

                    endpoint.update();
                }
            },
            cli::EndpointCommands::Headers {
                endpoint,
                add: add_list,
                remove: remove_list,
                list: should_list,
            } => {
                let mut endpoint = Endpoint::from_name(&endpoint);

                for key in remove_list {
                    endpoint.headers.remove(&key);
                }

                for header in add_list {
                    let splitted_item = header.splitn(2, ": ").collect::<Vec<&str>>();

                    if splitted_item.len() <= 1 {
                        panic!("Malformed header argument: {}", header);
                    }

                    let key = splitted_item[0];
                    let value = splitted_item[1];

                    endpoint.headers.insert(key.to_string(), value.to_string());
                }

                if should_list {
                    for (key, value) in endpoint.headers.iter() {
                        println!("{}: {}", key, value);
                    }
                }

                endpoint.update();
            }
            cli::EndpointCommands::Body {
                endpoint,
                stdin: expects_stdin,
                edit: should_edit,
                print: should_print,
            } => {
                let mut endpoint = Endpoint::from_name(&endpoint);

                if expects_stdin {
                    let mut input = String::new();

                    while let Ok(bytes) = std::io::stdin().read_line(&mut input) {
                        if bytes == 0 {
                            break;
                        }
                    }

                    endpoint.body = Body::from(input);
                }

                if should_edit {
                    let _ = std::process::Command::new(config.preferences.editor)
                        .arg(endpoint.dir().join("body.json"))
                        .status()
                        .expect("Failed to open editor");
                }

                if should_print {
                    while let Some(chunk) = endpoint.body.data().await {
                        stdout().write_all(&chunk.unwrap()).await.unwrap();
                    }
                }

                endpoint.update();
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
