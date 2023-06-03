mod cli;
mod config;
mod endpoint;
mod state;

use core::panic;
use std::{path::Path, process::exit};

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
        Commands::Send { endpoint } => {
            let endpoint = match endpoint {
                Some(name) => Endpoint::from_name(&name),
                None => Endpoint::from_state_or_exit(),
            };
            let req = endpoint.as_request().expect("Malformed request.");
            let client = Client::new();

            let mut res = client.request(req).await.unwrap();

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
            switch,
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

            if switch {
                if let Ok(()) = state::update_state(&config.name) {
                    println!("Switched to {} endpoint", config.name.green());
                } else {
                    eprintln!("Failed to switch to {} endpoint", config.name.red());
                    exit(1)
                }
            }

            config.write();
        }
        Commands::Use { endpoint } => {
            if !Path::new(".quartz")
                .join("endpoints")
                .join(Endpoint::name_to_dir(&endpoint))
                .is_dir()
            {
                eprintln!("Endpoint {} does not exist", &endpoint.red());
                exit(1);
            }

            if let Ok(()) = state::update_state(&endpoint) {
                println!("Switched to {} endpoint", endpoint.green());
            } else {
                panic!("Failed to switch to {} endpoint", endpoint.red());
            }
        }
        Commands::List => {
            let mut current = String::new();

            if let Some(endpoint) = Endpoint::from_state() {
                current = endpoint.name
            }

            if let Ok(files) = std::fs::read_dir(Path::new(".quartz").join("endpoints")) {
                for maybe_file in files {
                    if let Ok(file) = maybe_file {
                        let endpoint = Endpoint::from_name(file.file_name().to_str().unwrap());

                        if current == endpoint.name {
                            println!(
                                "* {: <5} {}",
                                endpoint.colored_method().bold(),
                                endpoint.name.green()
                            );
                        } else {
                            println!(
                                "  {: <5} {}",
                                endpoint.colored_method().bold(),
                                endpoint.name
                            );
                        }
                    }
                }
            }
        }
        Commands::Show { endpoint } => {
            let endpoint = match endpoint {
                Some(name) => Endpoint::from_name(&name),
                None => Endpoint::from_state_or_exit(),
            };

            println!("{}", endpoint.to_toml().unwrap());
        }
        Commands::Edit { endpoint, editor } => {
            let endpoint = match endpoint {
                Some(name) => Endpoint::from_name(&name),
                None => Endpoint::from_state_or_exit(),
            };

            let editor = match editor {
                Some(editor) => editor,
                None => config.preferences.editor,
            };

            let _ = std::process::Command::new(editor)
                .arg(endpoint.dir().join("config.toml"))
                .status()
                .expect("Failed to open editor");
        }
        Commands::Remove { endpoints } => {
            for endpoint in endpoints {
                let endpoint = Endpoint::from_name(&endpoint);

                if std::fs::remove_dir_all(endpoint.dir()).is_ok() {
                    println!("Deleted endpoint {}", endpoint.name);
                } else {
                    eprintln!("Failed to delete endpoint {}", endpoint.name);
                    exit(1);
                }
            }
        }
        Commands::Rename { endpoint, new_name } => {
            let mut endpoint = Endpoint::from_name(&endpoint);
            let src = endpoint.dir();

            endpoint.name = new_name.clone();

            let dist = endpoint.dir();

            if let Ok(()) = std::fs::rename(src, dist) {
                endpoint.update();
            } else {
                eprintln!("Failed to rename endpoint");
                exit(1);
            }
        }
        Commands::Url { command } => match command {
            cli::EndpointUrlCommands::Get { endpoint } => {
                let endpoint = match endpoint {
                    Some(name) => Endpoint::from_name(&name),
                    None => Endpoint::from_state_or_exit(),
                };

                println!("{}", endpoint.url);
            }
            cli::EndpointUrlCommands::Set { endpoint, url } => {
                let mut endpoint = match endpoint {
                    Some(name) => Endpoint::from_name(&name),
                    None => Endpoint::from_state_or_exit(),
                };

                endpoint.url = url;

                endpoint.update();
            }
        },
        Commands::Method { command } => match command {
            cli::EndpointMethodCommands::Get { endpoint } => {
                let endpoint = match endpoint {
                    Some(name) => Endpoint::from_name(&name),
                    None => Endpoint::from_state_or_exit(),
                };

                println!("{}", endpoint.method);
            }
            cli::EndpointMethodCommands::Set { endpoint, method } => {
                let mut endpoint = match endpoint {
                    Some(name) => Endpoint::from_name(&name),
                    None => Endpoint::from_state_or_exit(),
                };

                endpoint.method = method.to_uppercase();

                endpoint.update();
            }
        },
        Commands::Headers {
            endpoint,
            add: add_list,
            remove: remove_list,
            list: should_list,
        } => {
            let mut endpoint = match endpoint {
                Some(name) => Endpoint::from_name(&name),
                None => Endpoint::from_state_or_exit(),
            };

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
        Commands::Body {
            endpoint,
            stdin: expects_stdin,
            edit: should_edit,
            print: should_print,
        } => {
            let mut endpoint = match endpoint {
                Some(name) => Endpoint::from_name(&name),
                None => Endpoint::from_state_or_exit(),
            };

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
