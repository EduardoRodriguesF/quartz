mod cli;
mod config;
mod context;
mod endpoint;
mod history;
mod state;

use core::panic;
use std::{
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use context::Context;
use endpoint::{Endpoint, EndpointHandle};
use history::{RequestHistory, RequestHistoryEntry};
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
    let mut config = Config::parse();

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(config.ui.colors);

    std::panic::set_hook(Box::new(|info| {
        if let Some(payload) = info.payload().downcast_ref::<String>() {
            eprintln!("{}: {payload}", "error".red().bold());
        } else {
            eprintln!("{}: {info}", "error".red().bold());
        }
    }));

    match args.command {
        Commands::Init { directory } => {
            let directory = directory.unwrap_or(Path::new(".").to_path_buf());
            let quartz_dir = directory.join(".quartz");

            if quartz_dir.exists() {
                panic!(
                    "quartz already initialized at {}",
                    directory.to_string_lossy()
                );
            }

            if std::fs::create_dir(&quartz_dir).is_err() {
                panic!("failed to initialize quartz project");
            };

            let ensure_dirs = vec![
                "endpoints",
                "user",
                "user/history",
                "user/state",
                "contexts",
            ];

            for dir in ensure_dirs {
                if std::fs::create_dir(quartz_dir.join(PathBuf::from_str(dir).unwrap())).is_err() {
                    panic!("failed to create {} directory", dir);
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
                panic!("failed to create default context");
            }

            config.write().expect("failed to save configuration file");
        }
        Commands::Send { handle } => {
            let specification = match !handle.is_empty() {
                true => EndpointHandle::from_handle(handle),
                false => EndpointHandle::from_state_or_exit(),
            };
            let mut history_entry = RequestHistoryEntry::new();
            let context = Context::parse(&State::Context.get().unwrap_or(String::from("default")));

            let mut endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    panic!("no endpoint at {}", specification.head().red());
                })
                .clone();

            history_entry.endpoint(&endpoint);

            if let Ok(context) = context {
                endpoint.apply_context(&context);
                history_entry.context(&context);
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

            history_entry
                .body(&bytes)
                .status(res.status().as_u16())
                .path(specification.path)
                .duration(duration.as_millis() as u64);

            println!("Status: {}", res.status());
            println!("Duration: {}ms", duration.as_millis());
            println!("Size: {} bytes", size);

            let _ = stdout().write_all(&bytes).await;
            let _ = history_entry.write();
        }
        Commands::Create {
            handle,
            url: maybe_url,
            method: maybe_method,
            header,
            switch,
        } => {
            if handle.is_empty() {
                panic!("missing endpoint handle");
            }

            let mut handle = EndpointHandle::from_handle(handle);

            if handle.exists() {
                panic!("endpoint already exists");
            }

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
                if let Ok(()) = State::Endpoint.set(&handle.path.join(" ")) {
                    println!("Switched to {} endpoint", handle.head().green());
                } else {
                    panic!("failed to switch to {} endpoint", handle.head().red());
                }
            }

            handle.endpoint = Some(endpoint);
            handle.write();
        }
        Commands::Use { handle } => {
            let specification = EndpointHandle::from_handle(handle);

            if !specification.dir().exists() {
                panic!("endpoint does not exist");
            }

            if let Ok(()) = State::Endpoint.set(&specification.path.join(" ")) {
                println!("switched to {} endpoint", specification.head().green());
            } else {
                panic!(
                    "Failed to switch to {} endpoint",
                    specification.head().red()
                );
            }
        }
        Commands::Status { command } => match command {
            cli::StatusCommands::Endpoint => {
                if let Ok(endpoint) = State::Endpoint.get() {
                    println!("{}", endpoint);
                }
            }
            cli::StatusCommands::Context => {
                println!("{}", State::Context.get().unwrap_or("default".into()));
            }
        },
        Commands::List { depth: max_depth } => {
            let max_depth = max_depth.unwrap_or(999) as i16;
            let mut current = PathBuf::new();

            if let Some(specification) = EndpointHandle::from_state() {
                current = specification.dir()
            }

            // This code is a mess.
            // I'm sorry.
            // It will be refactored sometime.
            struct TraverseEndpoints<'s> {
                f: &'s dyn Fn(&TraverseEndpoints, Vec<EndpointHandle>),
            }
            let traverse_handles = TraverseEndpoints {
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
                                    println!();
                                }

                                (recurse.f)(recurse, children);
                            } else {
                                println!("{}", " +".dimmed());
                            }
                        } else {
                            println!();
                        }
                    }
                },
            };

            (traverse_handles.f)(&traverse_handles, vec![EndpointHandle::QUARTZ]);
        }
        Commands::Show { handle } => {
            let specification = match !handle.is_empty() {
                true => EndpointHandle::from_handle(handle),
                false => EndpointHandle::from_state_or_exit(),
            };

            if let Some(endpoint) = specification.endpoint {
                println!("{}", endpoint.to_toml().unwrap());
            } else {
                println!("No endpoint configured");
            }
        }
        Commands::Edit { editor } => {
            let specification = EndpointHandle::from_state_or_exit();

            let editor = match editor {
                Some(editor) => editor,
                None => config.preferences.editor,
            };

            let _ = std::process::Command::new(editor)
                .arg(specification.dir().join("endpoint.toml"))
                .status()
                .expect("Failed to open editor");
        }
        Commands::Remove { handle } => {
            let specification = EndpointHandle::from_handle(handle);

            if std::fs::remove_dir_all(specification.dir()).is_ok() {
                println!("Deleted endpoint {}", specification.head());
            } else {
                panic!("failed to delete endpoint {}", specification.head());
            }
        }
        Commands::Url { command } => match command {
            cli::EndpointUrlCommands::Get => {
                let specification = EndpointHandle::from_state_or_exit();
                let endpoint = specification.endpoint.as_ref().unwrap_or_else(|| {
                    panic!("no endpoint at {}", specification.head().red());
                });

                println!("{}", endpoint.url);
            }
            cli::EndpointUrlCommands::Set { url } => {
                let mut specification = EndpointHandle::from_state_or_exit();
                let mut endpoint = specification
                    .endpoint
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("no endpoint at {}", specification.head().red());
                    })
                    .clone();

                endpoint.url = url;

                specification.endpoint = Some(endpoint);
                specification.update();
            }
        },
        Commands::Method { command } => match command {
            cli::EndpointMethodCommands::Get => {
                let specification = EndpointHandle::from_state_or_exit();
                let endpoint = specification.endpoint.as_ref().unwrap_or_else(|| {
                    panic!("no endpoint at {}", specification.head().red());
                });

                println!("{}", endpoint.method);
            }
            cli::EndpointMethodCommands::Set { method } => {
                let mut specification = EndpointHandle::from_state_or_exit();
                let mut endpoint = specification
                    .endpoint
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("no endpoint at {}", specification.head().red());
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
            let mut specification = EndpointHandle::from_state_or_exit();
            let mut endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    panic!("no endpoint at {}", specification.head().red());
                })
                .clone();

            for key in remove_list {
                endpoint.headers.remove(&key);
            }

            for header in add_list {
                let splitted_item = header.splitn(2, ": ").collect::<Vec<&str>>();

                if splitted_item.len() <= 1 {
                    panic!("malformed header argument: {}", header);
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
            let mut specification = EndpointHandle::from_state_or_exit();
            let endpoint = specification
                .endpoint
                .as_ref()
                .unwrap_or_else(|| {
                    panic!("no endpoint at {}", specification.head().red());
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
        Commands::History { max_count, date } => {
            let mut history = RequestHistory::new().unwrap();
            let mut count = 0;
            let max_count = max_count.unwrap_or(usize::MAX);
            let date = &date.unwrap_or("%a %b %d %H:%M:%S %Y".into());

            while let Some(entry) = history.next() {
                if count >= max_count {
                    break;
                }

                count += 1;
                let endpoint = entry.endpoint.as_ref().unwrap();

                if count != 1 {
                    println!();
                }

                // Heading line
                print!(
                    "{} {}",
                    endpoint.colored_method(),
                    entry.path.join(" ").yellow(),
                );

                if let Some(status) = &entry.status {
                    if let Ok(status) = hyper::StatusCode::from_u16(*status) {
                        print!(" -> {}", status);
                    }
                }

                // End of heading line
                println!();

                let context_name: String = match &entry.context {
                    Some(context) => context.name.clone(),
                    None => "none".into(),
                };

                println!("Url: {}", endpoint.url);
                println!("Context: {}", context_name);
                println!(
                    "Date: {}",
                    entry.format_time(date).unwrap_or("Unknown".into())
                );

                println!();
                println!("{}", entry.body);
            }
        }
        Commands::Variable {
            get: maybe_get,
            set: maybe_set,
            edit: should_edit,
            list: should_list,
        } => {
            let state = State::Context.get().unwrap_or("default".into());

            let mut context = Context::parse(&state).unwrap_or_else(|_| {
                panic!("failed to parse {} context", state);
            });

            if let Some(var) = maybe_get {
                if let Some(value) = context.variables.get(&var) {
                    println!("{}", value);
                } else {
                    panic!("variable {} does not exist", var);
                }
            }

            if should_edit {
                let _ = std::process::Command::new(config.preferences.editor)
                    .arg(context.dir().join("variables.toml"))
                    .status()
                    .expect("Failed to open editor");
            }

            if let Some(set) = maybe_set {
                let split_set = set.splitn(2, '=').collect::<Vec<&str>>();

                if split_set.len() != 2 {
                    panic!(
                        "malformed argument. Try using {}",
                        "quartz variable --set <key>=<value>".green()
                    );
                }

                let key = split_set[0];
                let value = split_set[1].trim_matches('\'').trim_matches('\"');

                context.variables.insert(key.to_string(), value.to_string());
            }

            if should_list {
                if let Ok(list) = toml::ser::to_string(&context.variables) {
                    println!("{}", list);
                } else {
                    panic!("failed to list variables");
                }
            }

            let _ = context.update();
        }
        Commands::Context { command } => match command {
            cli::ContextCommands::Create { name, copy } => {
                let context = match copy {
                    Some(copy_from) => {
                        let mut context = Context::parse(&copy_from).unwrap_or_else(|_| {
                            panic!("no context named {} to copy from.", copy_from.red());
                        });

                        context.name = name.clone();
                        context
                    }
                    None => Context::new(&name),
                };

                if context.exists() {
                    panic!("A context named {} already exists", name.red());
                }

                if context.write().is_err() {
                    panic!("Failed to create {} context", name);
                }
            }
            cli::ContextCommands::Use { context } => {
                let context = Context::new(&context);

                if !context.exists() {
                    panic!("context {} does not exist", context.name.red());
                }

                if let Ok(()) = State::Context.set(&context.name) {
                    println!("Switched to {} context", context.name.green());
                } else {
                    panic!("failed to switch to {} context", context.name.red());
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
                    panic!("context {} does not exist", context.name.red());
                }

                if std::fs::remove_dir_all(context.dir()).is_ok() {
                    println!("Deleted {} context", context.name.green());
                } else {
                    panic!("failed to delete {} context", context.name.red());
                }
            }
        },
        Commands::Config { command } => match command {
            cli::ConfigCommands::Get { key } => {
                let value: String = match key.as_str() {
                    "preferences.editor" => config.preferences.editor,
                    "ui.colors" => config.ui.colors.to_string(),
                    _ => panic!("invalid key"),
                };

                println!("{value}");
            }
            cli::ConfigCommands::Edit => {
                let _ = std::process::Command::new(config.preferences.editor)
                    .arg(Config::filepath().to_str().unwrap())
                    .status()
                    .expect("Failed to open editor");
            }
            cli::ConfigCommands::Set { key, value } => {
                match key.as_str() {
                    "preferences.editor" => config.preferences.editor = value,
                    "ui.colors" => {
                        let value = match value.as_str() {
                            "true" => true,
                            _ => false,
                        };

                        config.ui.colors = value
                    }
                    _ => panic!("invalid key"),
                };

                if config.write().is_err() {
                    panic!("failed to save config change");
                }
            }
            cli::ConfigCommands::List => {
                let content = toml::to_string(&config).expect("could not parse configuration file");

                println!("{content}");
            }
        },
    }
}
