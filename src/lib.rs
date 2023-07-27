pub mod cli;
pub mod config;
pub mod context;
pub mod endpoint;
pub mod history;
pub mod state;

use colored::Colorize;

use config::Config;
use context::Context;
use endpoint::{Endpoint, EndpointHandle};
use state::{State, StateField};

pub struct CtxArgs {
    pub from_handle: Option<String>,
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

        let endpoint = specification
            .endpoint
            .as_ref()
            .unwrap_or_else(|| {
                panic!("no endpoint at {}", specification.handle().red());
            })
            .clone();

        (specification, endpoint)
    }

    pub fn require_context(&self) -> Context {
        let state = self.state.get(StateField::Context);

        match state {
            Ok(state) => Context::parse(&state)
                .unwrap_or_else(|_| panic!("could not resolve {} context", state.red())),
            Err(..) => Context::default(),
        }
    }
}
