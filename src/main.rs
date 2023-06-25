mod cli;
mod config;
mod context;
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
use context::Context;
use endpoint::{Endpoint, Specification};
use hyper::{
    body::{Bytes, HttpBody},
    Body, Client,
};
use state::State;
use tokio::io::{stdout, AsyncWriteExt as _};
use tokio::time::Instant;

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

            let ensure_dirs = vec!["endpoints", "user", "user/log", "user/state", "contexts"];

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

            if Context::default().write().is_err() {
                eprintln!("Failed to create default context");
                exit(1);
            }
        }
        Commands::Send => {
            let specification = Specification::from_state_or_exit();
            let context = Context::parse(&State::Context.get().unwrap_or(String::from("default")));
            let mut endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                })
                .clone();

            if let Ok(context) = context {
                endpoint.apply_context(&context);
            }

            let req = endpoint
                .into_request(&specification)
                .expect("Malformed request.");

            let client = {
                let https = hyper_tls::HttpsConnector::new();
                Client::builder().build(https)
            };

            let start = Instant::now();
            let mut res = client.request(req).await.unwrap();
            let duration = start.elapsed();

            let mut bytes = Bytes::new();
            let mut size = 0;

            while let Some(chunk) = res.data().await {
                if let Ok(chunk) = chunk {
                    size = chunk.len();
                    bytes = chunk;
                }
            }

            println!("Status: {}", res.status());
            println!("Duration: {}ms", duration.as_millis());
            println!("Size: {} bytes", size);

            let _ = stdout().write_all(&bytes).await;
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
                if let Ok(()) = State::Endpoint.set(&specification.path.join(" ")) {
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

            if !specification.dir().exists() {
                eprintln!("Endpoint does not exist");
                exit(1);
            }

            if let Ok(()) = State::Context.set(&specification.path.join(" ")) {
                println!("switched to {} endpoint", specification.head().green());
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
                .arg(specification.dir().join("endpoint.toml"))
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
            let endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    eprintln!("No endpoint at {}", specification.head().red());
                    exit(1);
                })
                .clone();

            let mut body = endpoint.body(&specification);

            if expects_stdin {
                let mut input = String::new();

                while let Ok(bytes) = std::io::stdin().read_line(&mut input) {
                    if bytes == 0 {
                        break;
                    }
                }

                body = Body::from(input);
            }

            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(specification.dir().join("body.json"))
            {
                while let Some(chunk) = body.data().await {
                    let _ = file.write_all(&chunk.unwrap());
                }
            }

            if should_edit {
                let _ = std::process::Command::new(config.preferences.editor)
                    .arg(specification.dir().join("body.json"))
                    .status()
                    .expect("Failed to open editor");
            }

            if should_print {
                if let Some(chunk) = endpoint.body(&specification).data().await {
                    stdout().write_all(&chunk.unwrap()).await.unwrap();
                }
            }

            specification.endpoint = Some(endpoint);
            specification.update();
        }
        Commands::Variable {
            get: maybe_get,
            set: maybe_set,
            edit: should_edit,
            list: should_list,
        } => {
            let mut context = Context::parse("default").unwrap_or_else(|_| {
                eprintln!("Failed to parse {} context", "default".red());
                exit(1);
            });

            if let Some(var) = maybe_get {
                if let Some(value) = context.variables.get(&var) {
                    println!("{}", value);
                } else {
                    eprintln!("Variable {} does not exist", var.red());
                    exit(1);
                }
            }

            if should_edit {
                let _ = std::process::Command::new(config.preferences.editor)
                    .arg(context.dir().join("variables.toml"))
                    .status()
                    .expect("Failed to open editor");
            }

            if let Some(set) = maybe_set {
                let split_set = set.splitn(2, "=").collect::<Vec<&str>>();

                if split_set.len() != 2 {
                    eprintln!(
                        "Malformed argument. Try using {}",
                        "quartz context variable --set <key>=<value>".green()
                    );
                    exit(1);
                }

                let key = split_set[0];
                let value = split_set[1];

                context.variables.insert(key.to_string(), value.to_string());
            }

            if should_list {
                if let Ok(list) = toml::ser::to_string(&context.variables) {
                    println!("{}", list);
                } else {
                    eprintln!("Failed to list variables");
                    exit(1);
                }
            }

            let _ = context.update();
        }
        Commands::Context { command } => match command {
            cli::ContextCommands::Create { name, copy } => {
                let context = match copy {
                    Some(copy_from) => {
                        let mut context = Context::parse(&copy_from).unwrap_or_else(|_| {
                            eprintln!("No context named {} to copy from.", copy_from.red());
                            exit(1);
                        });

                        context.name = name.clone();
                        context
                    }
                    None => Context::new(&name),
                };

                if context.exists() {
                    eprintln!("A context named {} already exists", name.red());
                    exit(1);
                }

                if context.write().is_err() {
                    eprintln!("Failed to create {} context", name);
                    exit(1);
                }
            }
            cli::ContextCommands::Use { context } => {
                let context = Context::new(&context);

                if !context.exists() {
                    eprintln!("Context {} does not exist", context.name.red());
                    exit(1);
                }

                if let Ok(()) = State::Context.set(&context.name) {
                    println!("Switched to {} context", context.name.green());
                } else {
                    panic!("Failed to switch to {} endpoint", context.name.red());
                }
            }
            cli::ContextCommands::List => {
                if let Ok(entries) = std::fs::read_dir(Path::new(".quartz").join("contexts")) {
                    for entry in entries {
                        let bytes = entry.unwrap().file_name();
                        let context_name = bytes.to_str().unwrap();

                        let state = State::Context.get().unwrap_or(String::from("default"));

                        if state == context_name {
                            println!("* {}", context_name.green());
                        } else {
                            println!("  {}", context_name);
                        }
                    }
                }
            }
            cli::ContextCommands::Remove { context } => {
                let context = Context::new(&context);

                if !context.exists() {
                    eprintln!("Context {} does not exist", context.name.red());
                    exit(1);
                }

                if std::fs::remove_dir_all(context.dir()).is_ok() {
                    eprintln!("Deleted {} context", context.name.green());
                } else {
                    eprintln!("Failed to delete {} context", context.name.red());
                    exit(1);
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
