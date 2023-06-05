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
use endpoint::{Endpoint, Specification};
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
            let specification = Specification::from_state_or_exit();
            let endpoint = specification.endpoint.as_ref().unwrap_or_else(|| {
                eprintln!("No endpoint at {}", specification.head().red());
                exit(1);
            });

            let req = endpoint.as_request().expect("Malformed request.");
            let client = Client::new();

            let mut res = client.request(req).await.unwrap();

            println!("Status: {}", res.status());

            while let Some(chunk) = res.data().await {
                stdout().write_all(&chunk.unwrap()).await.unwrap();
            }
        }
        Commands::Create {
            specs,
            url: maybe_url,
            method: maybe_method,
            header,
            switch,
        } => {
            let mut specification = Specification {
                path: specs,
                endpoint: None,
            };

            let mut endpoint = Endpoint::new();

            for item in header {
                let splitted_item = item.splitn(2, ": ").collect::<Vec<&str>>();

                if splitted_item.len() <= 1 {
                    panic!("Malformed header argument: {}", item);
                }

                let key = splitted_item[0];
                let value = splitted_item[1];

                endpoint.headers.insert(key.to_string(), value.to_string());
            }

            if let Some(url) = maybe_url {
                endpoint.url = url;
            }

            if let Some(method) = maybe_method {
                endpoint.method = method;
            }

            if switch {
                if let Ok(()) = state::update_state(&specification.path.join(" ")) {
                    println!("Switched to {} endpoint", specification.head().green());
                } else {
                    eprintln!(
                        "Failed to switch to {} endpoint",
                        specification.head().red()
                    );
                    exit(1)
                }
            }

            specification.endpoint = Some(endpoint);
            specification.write();
        }
        Commands::Use { endpoint } => {
            let specification = Specification::from_nesting(endpoint);

            if let Ok(()) = state::update_state(&specification.path.join(" ")) {
                println!("Switched to {} endpoint", specification.head().green());
            } else {
                panic!(
                    "Failed to switch to {} endpoint",
                    specification.head().red()
                );
            }
        }
        Commands::List { depth: max_depth } => {
            let max_depth = max_depth.unwrap_or(999) as i16;
            let mut current = PathBuf::new();

            if let Some(specification) = Specification::from_state() {
                current = specification.dir()
            }

            // This code is a mess.
            // I'm sorry.
            // It will be refactored sometime.
            struct TraverseEndpoints<'s> {
                f: &'s dyn Fn(&TraverseEndpoints, Vec<Specification>),
            }
            let traverse_specs = TraverseEndpoints {
                f: &|recurse, specifications| {
                    for spec in specifications {
                        let depth = (spec.path.len() as i16 - 1).max(0);
                        let children = spec.children();

                        let mut padding = 0;
                        while padding < depth {
                            print!("       ");
                            padding += 1;
                        }

                        if let Some(endpoint) = spec.endpoint.as_ref() {
                            if current == spec.dir() {
                                print!(
                                    "*  {: <5} {}",
                                    endpoint.colored_method().bold(),
                                    spec.head().green()
                                );
                            } else {
                                print!(
                                    "   {: <5} {}",
                                    endpoint.colored_method().bold(),
                                    spec.head()
                                );
                            }
                        } else if !spec.path.is_empty() {
                            print!("   {}", spec.head());
                        }

                        if !children.is_empty() {
                            if depth < max_depth {
                                // Avoid extra newline from Specification::QUARTZ usage
                                if !spec.path.is_empty() {
                                    print!("\n");
                                }

                                (recurse.f)(recurse, children);
                            } else {
                                print!("{}\n", " +".dimmed());
                            }
                        } else {
                            print!("\n");
                        }
                    }
                },
            };

            (traverse_specs.f)(&traverse_specs, vec![Specification::QUARTZ]);
        }
        Commands::Show => {
            let specification = Specification::from_state_or_exit();

            if let Some(endpoint) = specification.endpoint {
                println!("{}", endpoint.to_toml().unwrap());
            } else {
                println!("No endpoint configured");
            }
        }
        Commands::Edit { editor } => {
            let specification = Specification::from_state_or_exit();

            let editor = match editor {
                Some(editor) => editor,
                None => config.preferences.editor,
            };

            let _ = std::process::Command::new(editor)
                .arg(specification.dir().join("config.toml"))
                .status()
                .expect("Failed to open editor");
        }
        Commands::Remove { endpoint } => {
            let specification = Specification::from_nesting(endpoint);

            if std::fs::remove_dir_all(specification.dir()).is_ok() {
                println!("Deleted endpoint {}", specification.head());
            } else {
                eprintln!("Failed to delete endpoint {}", specification.head());
                exit(1);
            }
        }
        Commands::Url { command } => match command {
            cli::EndpointUrlCommands::Get => {
                let specification = Specification::from_state_or_exit();
                let endpoint = specification.endpoint.as_ref().unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                });

                println!("{}", endpoint.url);
            }
            cli::EndpointUrlCommands::Set { url } => {
                let mut specification = Specification::from_state_or_exit();
                let mut endpoint = specification
                    .endpoint
                    .as_ref()
                    .unwrap_or_else(|| {
                        eprintln!("No endpoint at {}", specification.head().red());
                        exit(1);
                    })
                    .clone();

                endpoint.url = url;

                specification.endpoint = Some(endpoint);
                specification.update();
            }
        },
        Commands::Method { command } => match command {
            cli::EndpointMethodCommands::Get => {
                let specification = Specification::from_state_or_exit();
                let endpoint = specification.endpoint.as_ref().unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                });

                println!("{}", endpoint.method);
            }
            cli::EndpointMethodCommands::Set { method } => {
                let mut specification = Specification::from_state_or_exit();
                let mut endpoint = specification
                    .endpoint
                    .as_ref()
                    .unwrap_or_else(|| {
                        eprintln!("No endpoint at {}", specification.head().red());
                        exit(1);
                    })
                    .clone();

                endpoint.method = method.to_uppercase();

                specification.endpoint = Some(endpoint);
                specification.update();
            }
        },
        Commands::Headers {
            add: add_list,
            remove: remove_list,
            list: should_list,
        } => {
            let mut specification = Specification::from_state_or_exit();
            let mut endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                })
                .clone();

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

            specification.endpoint = Some(endpoint);
            specification.update();
        }
        Commands::Body {
            stdin: expects_stdin,
            edit: should_edit,
            print: should_print,
        } => {
            let mut specification = Specification::from_state_or_exit();
            let mut endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                })
                .clone();

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
                    .arg(specification.dir().join("body.json"))
                    .status()
                    .expect("Failed to open editor");
            }

            if should_print {
                while let Some(chunk) = endpoint.body.data().await {
                    stdout().write_all(&chunk.unwrap()).await.unwrap();
                }
            }

            specification.endpoint = Some(endpoint);
            specification.update();
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
