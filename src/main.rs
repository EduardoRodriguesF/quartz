mod cli;
mod config;
mod endpoint;
mod state;

use core::panic;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::exit,
    str::FromStr,
};

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
        Commands::Init { directory } => {
            let directory = directory.unwrap_or(Path::new(".").to_path_buf());
            let quartz_dir = directory.join(".quartz");

            if quartz_dir.exists() {
                eprintln!(
                    "quartz already initialized at {}",
                    directory.to_str().unwrap().red()
                );
                exit(1);
            }

            if std::fs::create_dir(&quartz_dir).is_err() {
                eprintln!("Failed to initialize quartz project");
                exit(1);
            };

            let ensure_dirs = vec!["endpoints", "user", "user/log", "user/state"];

            for dir in ensure_dirs {
                if std::fs::create_dir(quartz_dir.join(PathBuf::from_str(dir).unwrap())).is_err() {
                    eprintln!("Failed to create {} directory", dir.red());
                    exit(1);
                }
            }

            if directory.join(".git").exists() {
                println!("Git detected");
                println!("Adding user files to {}", ".gitignore".green());

                if let Ok(mut gitignore) = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(directory.join(".gitignore"))
                {
                    let _ = gitignore.write("\n# Quartz\n.quartz/user".as_bytes());
                }
            }
        }
        Commands::Send => {
            let endpoint = Endpoint::from_state_or_exit();

            let req = endpoint.as_request().expect("Malformed request.");
            let client = Client::new();

            let mut res = client.request(req).await.unwrap();

            println!("Status: {}", res.status());

            while let Some(chunk) = res.data().await {
                stdout().write_all(&chunk.unwrap()).await.unwrap();
            }
        }
        Commands::Create {
            mut specs,
            url: maybe_url,
            method: maybe_method,
            header,
            switch,
        } => {
            let mut config = Endpoint::new(&specs.last().unwrap());
            specs.pop();

            config.parents = specs;

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
                if let Ok(()) = state::update_state(&config.nesting().join(" ")) {
                    println!("Switched to {} endpoint", config.name.green());
                } else {
                    eprintln!("Failed to switch to {} endpoint", config.name.red());
                    exit(1)
                }
            }

            config.write();
        }
        Commands::Use { endpoint } => {
            let endpoint = Endpoint::from_nesting(endpoint).expect("Endpoint does not exist");

            if let Ok(()) = state::update_state(&endpoint.nesting().join(" ")) {
                println!("Switched to {} endpoint", endpoint.name.green());
            } else {
                panic!("Failed to switch to {} endpoint", endpoint.name.red());
            }
        }
        Commands::List { depth: max_depth } => {
            let max_depth = max_depth.unwrap_or(u16::MAX);
            let mut current = PathBuf::new();
            let dir = Path::new(".quartz").join("endpoints");

            if let Some(endpoint) = Endpoint::from_state() {
                current = endpoint.dir()
            }

            // This code is a mess.
            // I'm sorry.
            // It will be refactored sometime.
            struct TraverseEndpoints<'s> {
                f: &'s dyn Fn(&TraverseEndpoints, Vec<Endpoint>, u16),
            }
            let traverse_endpoints = TraverseEndpoints {
                f: &|recurse, endpoints, depth| {
                    for endpoint in endpoints {
                        let children = endpoint.children();

                        let mut padding = 0;
                        while padding < depth {
                            print!("       ");
                            padding += 1;
                        }

                        if current == endpoint.dir() {
                            print!(
                                "*  {: <5} {}",
                                endpoint.colored_method().bold(),
                                endpoint.name.green()
                            );
                        } else {
                            print!(
                                "   {: <5} {}",
                                endpoint.colored_method().bold(),
                                endpoint.name
                            );
                        }

                        if !children.is_empty() {
                            if depth < max_depth {
                                print!("\n");
                                (recurse.f)(recurse, children, depth + 1);
                            } else {
                                print!("{}\n", " +".dimmed());
                            }
                        } else {
                            print!("\n");
                        }
                    }
                },
            };

            if let Ok(paths) = std::fs::read_dir(dir) {
                let mut toplevel_endpoints = Vec::<Endpoint>::new();

                for path in paths {
                    if let Ok(endpoint) = Endpoint::from_dir(path.unwrap().path()) {
                        toplevel_endpoints.push(endpoint);
                    }
                }

                (traverse_endpoints.f)(&traverse_endpoints, toplevel_endpoints, 0);
            }
        }
        Commands::Show => {
            let endpoint =Endpoint::from_state_or_exit();

            println!("{}", endpoint.to_toml().unwrap());
        }
        Commands::Edit { editor } => {
            let endpoint = Endpoint::from_state_or_exit();

            let editor = match editor {
                Some(editor) => editor,
                None => config.preferences.editor,
            };

            let _ = std::process::Command::new(editor)
                .arg(endpoint.dir().join("config.toml"))
                .status()
                .expect("Failed to open editor");
        }
        Commands::Remove { endpoint } => {
            let endpoint = Endpoint::from_nesting(endpoint)
                .expect("Could not find endpoint");

            if std::fs::remove_dir_all(endpoint.dir()).is_ok() {
                println!("Deleted endpoint {}", endpoint.name);
            } else {
                eprintln!("Failed to delete endpoint {}", endpoint.name);
                exit(1);
            }
        }
        Commands::Url { command } => match command {
            cli::EndpointUrlCommands::Get => {
                let endpoint =Endpoint::from_state_or_exit();

                println!("{}", endpoint.url);
            }
            cli::EndpointUrlCommands::Set { url } => {
                let mut endpoint = Endpoint::from_state_or_exit();

                endpoint.url = url;

                endpoint.update();
            }
        },
        Commands::Method { command } => match command {
            cli::EndpointMethodCommands::Get => { let endpoint = Endpoint::from_state_or_exit();

                println!("{}", endpoint.method);
            }
            cli::EndpointMethodCommands::Set { method } => {
                let mut endpoint = Endpoint::from_state_or_exit();

                endpoint.method = method.to_uppercase();

                endpoint.update();
            }
        },
        Commands::Headers {
            add: add_list,
            remove: remove_list,
            list: should_list,
        } => {
            let mut endpoint = Endpoint::from_state_or_exit();

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
            stdin: expects_stdin,
            edit: should_edit,
            print: should_print,
        } => {
            let mut endpoint = Endpoint::from_state_or_exit();

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
