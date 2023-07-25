use colored::Colorize;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::context::Context;
use crate::state::{State, StateField};

#[derive(Debug)]
pub struct EndpointHandle {
    pub endpoint: Option<Endpoint>,

    /// List of ordered parent names
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// Query params.
    pub query: HashMap<String, String>,

    /// List of (key, value) pairs.
    pub headers: HashMap<String, String>,

    /// Variable values applied from a [`Context`]
    #[serde(skip_serializing, skip_deserializing)]
    pub variables: HashMap<String, String>,
}

impl EndpointHandle {
    /// Points to top-level quartz folder.
    ///
    /// This constant can be used to traverse through all specifications starting
    /// from the top one.
    pub const QUARTZ: Self = Self {
        path: vec![],
        endpoint: None,
    };

    pub fn from_handle<S>(handle: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut path = Path::new(".quartz").join("endpoints");
        let handle: Vec<String> = handle.as_ref().split('/').map(|s| s.to_string()).collect();

        for parent in &handle {
            let name = Endpoint::name_to_dir(parent);

            path.push(name);
        }

        let endpoint = match Endpoint::from_dir(path) {
            Ok(endpoint) => Some(endpoint),
            Err(_) => None,
        };

        Self {
            path: handle,
            endpoint,
        }
    }

    pub fn from_state(state: &State) -> Option<Self> {
        if let Ok(handle) = state.get(StateField::Endpoint) {
            if handle.is_empty() {
                return None;
            }

            return Some(EndpointHandle::from_handle(handle));
        }

        None
    }

    pub fn from_state_or_exit(state: &State) -> Self {
        match Self::from_state(state) {
            Some(endpoint) => endpoint,
            None => {
                panic!("no endpoint in use. Try {}", "quartz use <ENDPOINT>".cyan());
            }
        }
    }

    pub fn head(&self) -> String {
        self.path.last().unwrap_or(&String::new()).clone()
    }

    pub fn dir(&self) -> PathBuf {
        let mut result = Path::new(".quartz").join("endpoints");

        for parent in &self.path {
            let name = Endpoint::name_to_dir(parent);

            result = result.join(name);
        }

        result
    }

    pub fn handle(&self) -> String {
        self.path.join("/").to_string()
    }

    pub fn exists(&self) -> bool {
        self.dir().exists()
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self) {
        let mut dir = Path::new(".quartz").join("endpoints");
        for entry in &self.path {
            dir = dir.join(Endpoint::name_to_dir(entry));

            let _ = std::fs::create_dir(&dir);

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(dir.join("spec"))
                .unwrap();

            let _ = file.write_all(entry.as_bytes());
        }
        std::fs::create_dir_all(self.dir()).expect("Failed to create endpoint.");

        if let Some(endpoint) = &self.endpoint {
            let toml_content = endpoint.to_toml().expect("Failed to generate settings.");

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(self.dir().join("endpoint.toml"))
                .expect("Failed to open config file.");

            file.write_all(toml_content.as_bytes())
                .expect("Failed to write to config file.");
        }
    }

    /// Updates existing endpoint configuration file.
    // TODO: Only apply changes if a private flag is true.
    pub fn update(&self) {
        if let Some(endpoint) = &self.endpoint {
            let toml_content = endpoint.to_toml().expect("Failed to generate settings.");

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(self.dir().join("endpoint.toml"))
                .expect("Failed to open config file.");

            file.write_all(toml_content.as_bytes())
                .expect("Failed to write to config file.");
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.dir().join("spec"))
            .unwrap();

        let _ = file.write_all(self.head().as_bytes());
    }

    pub fn children(&self) -> Vec<EndpointHandle> {
        let mut list = Vec::<EndpointHandle>::new();

        if let Ok(paths) = std::fs::read_dir(self.dir()) {
            for path in paths {
                let path = path.unwrap().path();

                if !path.is_dir() {
                    continue;
                }

                let endpoint = match Endpoint::from_dir(path.clone()) {
                    Ok(endpoint) => Some(endpoint),
                    Err(_) => None,
                };

                if let Ok(vec) = std::fs::read(path.join("spec")) {
                    let spec = String::from_utf8(vec).unwrap_or_else(|_| {
                        panic!("failed to get endpoint specification");
                    });

                    let mut path = self.path.clone();
                    path.push(spec);

                    list.push(EndpointHandle { path, endpoint })
                }
            }
        }

        list
    }
}

impl Endpoint {
    pub fn new() -> Self {
        Self {
            method: String::from("GET"),
            ..Default::default()
        }
    }

    pub fn name_to_dir(name: &str) -> String {
        trim_newline(name.replace(['/', '\\'], "-"))
    }

    pub fn from_dir(dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(dir.join("endpoint.toml"))?;
        let content = String::from_utf8(bytes)?;

        let endpoint: Endpoint = toml::from_str(&content)?;

        Ok(endpoint)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }

    pub fn body(&self, spec: &EndpointHandle) -> Body {
        match std::fs::read(spec.dir().join("body.json")) {
            Ok(bytes) => {
                let mut content = String::from_utf8(bytes).unwrap();

                for (key, value) in &self.variables {
                    let key_match = format!("{{{{{}}}}}", key);

                    content = content.replace(&key_match, value);
                }

                content.into()
            }
            Err(_) => Body::empty(),
        }
    }

    pub fn apply_context(&mut self, context: &Context) {
        for (key, value) in &context.variables {
            let key_match = format!("{{{{{}}}}}", key);

            self.url = self.url.replace(&key_match, value);
            self.method = self.method.replace(&key_match, value);

            self.headers = self
                .headers
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();
        }

        self.variables = context.variables.clone();
    }

    /// Returns the a [`Request`] based of this [`EndpointConfig`].
    pub fn into_request(self, spec: &EndpointHandle) -> Result<Request<Body>, hyper::http::Error> {
        let mut builder = hyper::Request::builder().uri(&self.url);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        builder.body(self.body(spec))
    }

    pub fn colored_method(&self) -> colored::ColoredString {
        match self.method.as_str() {
            "GET" => self.method.blue(),
            "POST" => self.method.green(),
            "PUT" => self.method.yellow(),
            "PATCH" => self.method.yellow(),
            "DELETE" => self.method.red(),
            "OPTIONS" => self.method.cyan(),
            "HEAD" => self.method.cyan(),
            _ => self.method.white(),
        }
    }

    /// Return a query string based off of defined queries.
    ///
    /// ## Example
    ///
    /// A hash map composed of:
    ///
    /// ```toml
    /// [query]
    /// v = 9000
    /// fields = "lorem,ipsum"
    /// ```
    ///
    /// would return:
    ///
    ///     "v=9000&fields=lorem,ipsum"
    pub fn query_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in &self.query {
            result.push(format!("{key}={value}"));
        }

        result.join("&")
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            url: Default::default(),
            headers: Default::default(),
            variables: Default::default(),
            query: Default::default(),
        }
    }
}

fn trim_newline<S>(s: S) -> String
where
    S: Into<String> + std::fmt::Display,
{
    let mut s = s.to_string();

    while s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }

    s.trim().to_string()
}
