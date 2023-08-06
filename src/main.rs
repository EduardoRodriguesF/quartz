use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use colored::Colorize;
use hyper::{
    body::{Bytes, HttpBody},
    Body, Client,
};
use tokio::io::{stdout, AsyncWriteExt as _};
use tokio::time::Instant;

use quartz_cli::config::Config;
use quartz_cli::context::Context;
use quartz_cli::endpoint::{Endpoint, EndpointHandle};
use quartz_cli::history::{RequestHistory, RequestHistoryEntry};
use quartz_cli::state::StateField;
use quartz_cli::Ctx;
use quartz_cli::{
    cli::{self, Cli, Commands},
    CtxArgs,
};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let mut ctx = Ctx::new(CtxArgs {
        from_handle: args.from_handle,
        early_apply_context: args.apply_context,
    });

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(ctx.config.ui.colors);

    std::panic::set_hook(Box::new(|info| {
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
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

            ctx.config
                .write()
                .unwrap_or_else(|_| panic!("failed to save configuration file"));
        }
        Commands::Send => {
            let mut history_entry = RequestHistoryEntry::new();
            let (specification, mut endpoint) = ctx.require_endpoint();
            let context = ctx.require_context();
            history_entry.endpoint(&endpoint);

            endpoint.apply_context(&context);
            history_entry.context(&context);

            let req = endpoint
                .into_request(&specification)
                .unwrap_or_else(|_| panic!("malformed request"));

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
                .handle(specification.handle())
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
            query,
            header,
            switch,
        } => {
            if handle.is_empty() {
                panic!("missing endpoint handle");
            }

            let handle = EndpointHandle::from_handle(handle);

            if handle.exists() {
                panic!("endpoint already exists");
            }

            let mut endpoint = Endpoint::new();

            for item in header {
                let (key, value) = item
                    .split_once(": ")
                    .unwrap_or_else(|| panic!("malformed header argument: {}", item));

                endpoint.headers.insert(key.to_string(), value.to_string());
            }

            for item in query {
                let (key, value) = item
                    .split_once('=')
                    .unwrap_or_else(|| panic!("malformed query argument: {}", item));

                endpoint.query.insert(key.to_string(), value.to_string());
            }

            if let Some(url) = maybe_url {
                endpoint.url = url;
            }

            if let Some(method) = maybe_method {
                endpoint.method = method;
            }

            if switch {
                if let Ok(()) = StateField::Endpoint.set(&handle.path.join("/")) {
                    println!("Switched to {} endpoint", handle.handle().green());
                } else {
                    panic!("failed to switch to {} endpoint", handle.handle().red());
                }
            }

            handle.write();
            endpoint.write(handle);
        }
        Commands::Use { handle } => {
            let specification = EndpointHandle::from_handle(handle);

            if !specification.dir().exists() {
                panic!("endpoint does not exist");
            }

            if let Ok(()) = StateField::Endpoint.set(&specification.path.join("/")) {
                println!("Switched to {} endpoint", specification.handle().green());
            } else {
                panic!(
                    "failed to switch to {} endpoint",
                    specification.handle().red()
                );
            }
        }
        Commands::Status { command } => match command {
            cli::StatusCommands::Endpoint => {
                if let Ok(endpoint) = ctx.state.get(StateField::Endpoint) {
                    println!("{}", endpoint);
                }
            }
            cli::StatusCommands::Context => {
                println!(
                    "{}",
                    ctx.state
                        .get(StateField::Context)
                        .unwrap_or("default".into())
                );
            }
        },
        Commands::List { depth: max_depth } => {
            let max_depth = max_depth.unwrap_or(999) as i16;
            let mut current = PathBuf::new();

            if let Some(specification) = EndpointHandle::from_state(&ctx.state) {
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

                        if let Some(endpoint) = spec.endpoint() {
                            if current == spec.dir() {
                                print!(
                                    "*  {: >5} {}",
                                    endpoint.colored_method().bold(),
                                    spec.handle().green()
                                );
                            } else {
                                print!(
                                    "   {: >5} {}",
                                    endpoint.colored_method().bold(),
                                    spec.handle()
                                );
                            }
                        } else if !spec.path.is_empty() {
                            print!("   {: >5} {}", "---".dimmed(), spec.handle());
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
            ctx.args.from_handle = Some(handle).unwrap_or(ctx.args.from_handle);
            let (_, endpoint) = ctx.require_endpoint();

            println!("{}", endpoint.to_toml().unwrap());
        }
        Commands::Edit { editor } => {
            let specification = ctx.require_handle();

            if let Some(editor) = editor {
                ctx.config.preferences.editor = editor;
            }

            ctx.edit(specification.dir().join("endpoint.toml"), |c| {
                Ok(toml::de::from_str::<Endpoint>(&c)?)
            })
            .unwrap_or_else(|e| panic!("{}", e.to_string()));
        }
        Commands::Remove { handle } => {
            let specification = ctx.require_input_handle(&handle);

            if std::fs::remove_dir_all(specification.dir()).is_ok() {
                println!("Deleted endpoint {}", specification.handle());
            } else {
                panic!("failed to delete endpoint {}", specification.handle());
            }
        }
        Commands::Url { command } => match command {
            cli::EndpointUrlCommands::Get { full } => {
                let (_, endpoint) = ctx.require_endpoint();

                let mut url = endpoint.url.clone();

                if full {
                    url = endpoint
                        .full_url()
                        .unwrap_or_else(|_| panic!("invalid url"))
                        .to_string();
                }

                println!("{}", url);
            }
            cli::EndpointUrlCommands::Set { url } => {
                let (handle, mut endpoint) = ctx.require_endpoint();

                endpoint.url = url;

                endpoint.write(handle);
            }
        },
        Commands::Query { command } => match command {
            cli::EndpointQueryCommands::Get { key: maybe_key } => {
                let (_, endpoint) = ctx.require_endpoint();

                if let Some(key) = maybe_key {
                    let value = endpoint
                        .query
                        .get(&key)
                        .unwrap_or_else(|| panic!("no query param {} found", key.red()));

                    println!("{value}");
                } else {
                    // Display entire query
                    let query = endpoint.query_string();
                    println!("{query}");
                }
            }
            cli::EndpointQueryCommands::Set { query } => {
                let (handle, mut endpoint) = ctx.require_endpoint();

                let (key, value) = query
                    .split_once('=')
                    .unwrap_or_else(|| panic!("malformed query param: {}", query));

                endpoint.query.insert(key.to_string(), value.to_string());

                endpoint.write(handle);
            }
            cli::EndpointQueryCommands::Remove { key } => {
                let (handle, mut endpoint) = ctx.require_endpoint();

                endpoint.query.remove(&key);

                endpoint.write(handle);
            }
            cli::EndpointQueryCommands::List => {
                let (_, endpoint) = ctx.require_endpoint();

                for (key, value) in endpoint.query {
                    println!("{key}={value}");
                }
            }
        },
        Commands::Method { command } => match command {
            cli::EndpointMethodCommands::Get => {
                let (_, endpoint) = ctx.require_endpoint();

                println!("{}", endpoint.method);
            }
            cli::EndpointMethodCommands::Set { method } => {
                let (handle, mut endpoint) = ctx.require_endpoint();

                endpoint.method = method.to_uppercase();

                endpoint.write(handle);
            }
        },
        Commands::Header {
            set: set_list,
            remove: remove_list,
            list: should_list,
            get: maybe_get,
        } => {
            let (handle, mut endpoint) = ctx.require_endpoint();

            for key in remove_list {
                endpoint.headers.remove(&key);
            }

            for header in set_list {
                let (key, value) = header
                    .split_once(": ")
                    .unwrap_or_else(|| panic!("malformed header argument: {}", header));

                endpoint.headers.insert(key.to_string(), value.to_string());
            }

            if let Some(key) = maybe_get {
                if let Some(value) = endpoint.headers.get(&key) {
                    println!("{value}");
                }
            }

            if should_list {
                for (key, value) in endpoint.headers.iter() {
                    println!("{}: {}", key, value);
                }
            }

            endpoint.write(handle);
        }
        Commands::Body {
            stdin: expects_stdin,
            edit: should_edit,
            print: should_print,
        } => {
            let (handle, mut endpoint) = ctx.require_endpoint();

            let mut body = endpoint.body(&handle);

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
                .open(handle.dir().join("body.json"))
            {
                while let Some(chunk) = body.data().await {
                    let _ = file.write_all(&chunk.unwrap());
                }
            }

            if should_edit {
                let _ = std::process::Command::new(ctx.config.preferences.editor)
                    .arg(handle.dir().join("body.json"))
                    .status()
                    .unwrap_or_else(|_| panic!("failed to open editor"));
            }

            if should_print {
                if let Some(chunk) = endpoint.body(&handle).data().await {
                    stdout().write_all(&chunk.unwrap()).await.unwrap();
                }
            }

            endpoint.write(handle);
        }
        Commands::History { max_count, date } => {
            let history = RequestHistory::new().unwrap();
            let mut count = 0;
            let max_count = max_count.unwrap_or(usize::MAX);
            let date = &date.unwrap_or("%a %b %d %H:%M:%S %Y".into());

            for entry in history {
                if count >= max_count {
                    break;
                }

                count += 1;
                let endpoint = entry.endpoint.as_ref().unwrap();

                if count != 1 {
                    println!();
                }

                // Heading line
                print!("{} {}", endpoint.colored_method(), entry.handle.yellow(),);

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
            set: set_list,
            edit: should_edit,
            list: should_list,
        } => {
            let state = ctx
                .state
                .get(StateField::Context)
                .unwrap_or("default".into());

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

            if !set_list.is_empty() {
                for set in set_list {
                    let (key, value) = set.split_once('=').unwrap_or_else(|| {
                        panic!(
                            "malformed argument. Try using {}",
                            "quartz variable --set <key>=<value>".green()
                        )
                    });

                    let value = value.trim_matches('\'').trim_matches('\"');

                    context.variables.insert(key.to_string(), value.to_string());
                }

                context.update().expect("failed to update variables");
            }

            if should_list {
                if let Ok(list) = toml::ser::to_string(&context.variables) {
                    println!("{}", list);
                } else {
                    panic!("failed to list variables");
                }
            }

            if should_edit {
                ctx.edit(context.dir().join("variables.toml"), |c| {
                    Ok(toml::de::from_str::<HashMap<String, String>>(&c)?)
                })
                .unwrap_or_else(|e| panic!("{}", e.to_string()));
            }
        }
        Commands::Context { command } => match command {
            cli::ContextCommands::Create { name, copy } => {
                let context = match copy {
                    Some(copy_from) => {
                        let mut context = Context::parse(&copy_from).unwrap_or_else(|_| {
                            panic!("no context named {} to copy from", copy_from.red());
                        });

                        context.name = name.clone();
                        context
                    }
                    None => Context::new(&name),
                };

                if context.exists() {
                    panic!("a context named {} already exists", name.red());
                }

                if context.write().is_err() {
                    panic!("failed to create {} context", name);
                }
            }
            cli::ContextCommands::Use { context } => {
                let context = Context::new(&context);

                if !context.exists() {
                    panic!("context {} does not exist", context.name.red());
                }

                if let Ok(()) = StateField::Context.set(&context.name) {
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

                        let state = ctx
                            .state
                            .get(StateField::Context)
                            .unwrap_or(String::from("default"));

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
                    "preferences.editor" => ctx.config.preferences.editor,
                    "ui.colors" => ctx.config.ui.colors.to_string(),
                    _ => panic!("invalid key"),
                };

                println!("{value}");
            }
            cli::ConfigCommands::Edit => {
                let _ = std::process::Command::new(ctx.config.preferences.editor)
                    .arg(Config::filepath().to_str().unwrap())
                    .status()
                    .unwrap_or_else(|_| panic!("failed to open editor"));
            }
            cli::ConfigCommands::Set { key, value } => {
                match key.as_str() {
                    "preferences.editor" => ctx.config.preferences.editor = value,
                    "ui.colors" => ctx.config.ui.colors = matches!(value.as_str(), "true"),
                    _ => panic!("invalid key"),
                };

                if ctx.config.write().is_err() {
                    panic!("failed to save config change");
                }
            }
            cli::ConfigCommands::List => {
                let content = toml::to_string(&ctx.config)
                    .unwrap_or_else(|_| panic!("could not parse configuration file"));

                println!("{content}");
            }
        },
    }
}
