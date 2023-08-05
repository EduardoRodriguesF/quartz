pub mod cli;
pub mod config;
pub mod context;
pub mod endpoint;
pub mod history;
pub mod state;

use std::path::{Path, PathBuf};

use colored::Colorize;

use config::Config;
use context::Context;
use endpoint::{Endpoint, EndpointHandle};
use state::{State, StateField};

pub struct CtxArgs {
    pub from_handle: Option<String>,
    pub early_apply_context: bool,
}

pub struct Ctx {
    pub args: CtxArgs,
    pub config: Config,
    pub state: State,
}

impl Ctx {
    pub fn new(args: CtxArgs) -> Self {
        let config = Config::parse();
        let state = State {
            handle: args.from_handle.clone(),
        };

        Ctx {
            args,
            config,
            state,
        }
    }

    pub fn require_input_handle(&self, handle: &str) -> EndpointHandle {
        let result = EndpointHandle::from_handle(handle);

        if !result.exists() {
            panic!("could not find {} handle", handle.red());
        }

        result
    }

    pub fn require_handle(&self) -> EndpointHandle {
        if let Some(handle) = &self.args.from_handle {
            // Overwritten by argument
            return EndpointHandle::from_handle(handle);
        }

        let mut result = None;
        if let Ok(handle) = self.state.get(StateField::Endpoint) {
            if !handle.is_empty() {
                result = Some(EndpointHandle::from_handle(handle));
            }
        }

        match result {
            Some(handle) => handle,
            None => panic!("no handle in use. Try {}", "quartz use <HANDLE>".green()),
        }
    }

    pub fn require_endpoint(&self) -> (EndpointHandle, Endpoint) {
        let specification = self.require_handle();

        let mut endpoint = specification
            .endpoint
            .as_ref()
            .unwrap_or_else(|| {
                panic!("no endpoint at {}", specification.handle().red());
            })
            .clone();

        if self.args.early_apply_context {
            let context = self.require_context();
            endpoint.apply_context(&context);
        }

        (specification, endpoint)
    }

    pub fn require_context(&self) -> Context {
        let state = self
            .state
            .get(StateField::Context)
            .unwrap_or("default".into());

        Context::parse(&state)
            .unwrap_or_else(|_| panic!("could not resolve {} context", state.red()))
    }

    pub fn edit<T, F>(&self, path: PathBuf, validate: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(String) -> Result<T, Box<dyn std::error::Error>>,
    {
        let temp_path = Path::new(".quartz").join("user").join("EDIT.toml");
        std::fs::copy(&path, &temp_path).unwrap();

        let _ = std::process::Command::new(&self.config.preferences.editor)
            .arg(&temp_path)
            .status()
            .unwrap_or_else(|_| {
                panic!("failed to open editor: {}", &self.config.preferences.editor)
            });

        let content = std::fs::read_to_string(&temp_path)?;

        validate(content)?;

        std::fs::copy(&temp_path, path)?;
        std::fs::remove_file(temp_path)?;

        Ok(())
    }
}
