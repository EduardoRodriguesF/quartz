pub mod action;
pub mod cli;
pub mod config;
pub mod cookie;
pub mod endpoint;
pub mod env;
pub mod history;
pub mod snippet;
pub mod state;
pub mod validator;

use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{exit, Stdio};
use std::{collections::HashMap, ffi::OsString};

use colored::Colorize;

use config::Config;
use endpoint::{Endpoint, EndpointHandle};
use env::Env;
use state::{State, StateField};

pub type QuartzResult<T = (), E = Box<dyn std::error::Error>> = Result<T, E>;

pub enum QuartzCode {
    Success,
    Error,
}

#[derive(Debug)]
pub enum QuartzError {
    Internal,
}

impl Display for QuartzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuartzError::Internal => writeln!(f, "internal failure"),
        }
    }
}

impl Error for QuartzError {}

pub trait PairMap<'a, K = String, V = String>
where
    K: Eq + PartialEq + Hash + From<&'a str>,
    V: From<&'a str>,
{
    const NAME: &'static str = "key-value pair";
    const EXPECTED: &'static str = "<key>=<value>";

    /// Returns HashMap in the implementation struct.
    fn map(&mut self) -> &mut HashMap<K, V>;

    /// Breaks string into (key, value) tuple.
    fn pair(input: &'a str) -> Option<(K, V)> {
        let (key, value) = input.split_once('=')?;
        let value = value.trim_matches('\'').trim_matches('\"');

        Some((key.into(), value.into()))
    }

    /// Inserts key-value pair into map.
    fn set(&mut self, input: &'a str) {
        let (key, value) = Self::pair(input)
            .unwrap_or_else(|| panic!("malformed {}. Expected {}", Self::NAME, Self::EXPECTED));

        self.map().insert(key.into(), value.into());
    }
}

pub struct CtxArgs {
    pub from_handle: Option<String>,
    pub early_apply_environment: bool,
}

pub struct Ctx {
    pub args: CtxArgs,
    pub config: Config,
    pub state: State,
    path: PathBuf,
    code: QuartzCode,
}

impl Ctx {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    pub fn new(args: CtxArgs) -> QuartzResult<Self> {
        let config = Config::parse();
        let state = State {
            handle: args.from_handle.clone(),
        };

        let mut path = std::env::current_dir()?;
        loop {
            if path.join(".quartz").exists() {
                break;
            }

            if !path.pop() {
                panic!("could not find a quartz project");
            }
        }

        Ok(Ctx {
            args,
            config,
            state,
            path: path.join(".quartz"),
            code: QuartzCode::Success,
        })
    }

    pub fn require_input_handle(&self, handle: &str) -> EndpointHandle {
        let result = EndpointHandle::from(handle);

        if !result.exists(self) {
            panic!("could not find {} handle", handle.red());
        }

        result
    }

    pub fn require_handle(&self) -> EndpointHandle {
        if let Some(handle) = &self.args.from_handle {
            // Overwritten by argument
            return EndpointHandle::from(handle);
        }

        let mut result = None;
        if let Ok(handle) = self.state.get(self, StateField::Endpoint) {
            if !handle.is_empty() {
                result = Some(EndpointHandle::from(handle));
            }
        }

        match result {
            Some(handle) => handle,
            None => panic!("no handle in use. Try {}", "quartz use <HANDLE>".green()),
        }
    }

    pub fn require_endpoint(&self) -> (EndpointHandle, Endpoint) {
        let handle = self.require_handle();
        let endpoint = self.require_endpoint_from_handle(&handle);

        (handle, endpoint)
    }

    pub fn require_endpoint_from_handle(&self, handle: &EndpointHandle) -> Endpoint {
        let mut endpoint = handle.endpoint(self).unwrap_or_else(|| {
            panic!("no endpoint at {}", handle.handle().red());
        });

        if self.args.early_apply_environment {
            let env = self.require_env();
            endpoint.apply_env(&env);
        }

        endpoint
    }

    /// Returns current env.
    ///
    /// # Panics
    ///
    /// Program is terminated if it is unable to require it.
    pub fn require_env(&self) -> Env {
        let state = self
            .state
            .get(self, StateField::Env)
            .unwrap_or("default".into());

        Env::parse(self, &state)
            .unwrap_or_else(|_| panic!("could not resolve {} environment", state.red()))
    }

    /// Opens an editor to modified the specified file at `path` in a temporary file.
    ///
    /// After the program exits, `validate` function is ran on temporary file before moving it to
    /// the original file, effectively commiting the edits.
    ///
    /// If `validate` returns [`Err`], the temporary file is deleted while original file is preserved as is.
    ///
    /// # Arguments
    ///
    /// * `path` - A path slice to a file
    /// * `validate` - Validator method to ensure the edit can be saved without errors
    pub fn edit<F>(&self, path: &Path, validate: F) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        self.edit_with_extension::<F>(path, None, validate)
    }

    /// Opens an editor to modified the specified file at `path` with `extension` in a temporary file.
    ///
    /// After the program exits, `validate` function is ran on temporary file before moving it to
    /// the original file, effectively commiting the edits.
    ///
    /// If `validate` returns [`Err`], the temporary file is deleted while original file is preserved as is.
    ///
    /// # Arguments
    ///
    /// * `path` - A path slice to a file
    /// * `extension` - Which extension to create temporary file with
    /// * `validate` - Validator method to ensure the edit can be saved without errors
    pub fn edit_with_extension<F>(
        &self,
        path: &Path,
        extension: Option<&str>,
        validate: F,
    ) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        let mut temp_path = self.path().join("user").join(format!("EDIT"));

        let extension: Option<OsString> = {
            if let Some(extension) = extension {
                Some(OsString::from(extension))
            } else if let Some(extension) = path.extension() {
                Some(extension.to_os_string())
            } else {
                None
            }
        };

        if let Some(extension) = extension {
            temp_path.set_extension(extension);
        }

        if !path.exists() {
            std::fs::File::create(path)?;
        }

        std::fs::copy(path, &temp_path)?;

        let _ = std::process::Command::new(&self.config.preferences.editor)
            .arg(&temp_path)
            .status()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open editor: {}\n\n{}",
                    &self.config.preferences.editor, err
                );
            });

        let content = std::fs::read_to_string(&temp_path)?;

        if let Err(err) = validate(&content) {
            std::fs::remove_file(&temp_path)?;
            panic!("{}", err);
        }

        std::fs::rename(&temp_path, path)?;
        Ok(())
    }

    /// Open user's preferred pager with content.
    pub fn paginate(&self, input: &[u8]) -> QuartzResult {
        let mut child = std::process::Command::new(&self.config.preferences.pager)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open pager: {}\n\n{}",
                    &self.config.preferences.pager, err
                );
            });

        child.stdin.as_mut().unwrap().write_all(input.into())?;
        child.wait()?;

        Ok(())
    }

    pub fn user_agent() -> String {
        let mut agent = String::from("quartz/");
        agent.push_str(Ctx::VERSION);

        agent
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn code(&mut self, value: QuartzCode) {
        self.code = value;
    }

    pub fn exit(self) {
        exit(self.code as i32);
    }
}
