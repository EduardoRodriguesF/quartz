use colored::Colorize;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub name: String,
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// List of (key, value) pairs.
    pub headers: HashMap<String, String>,

    #[serde(skip_serializing, skip_deserializing)]
    pub body: Body,

    /// List of ordered parent names
    #[serde(skip_serializing, skip_deserializing)]
    pub parents: Vec<String>,
}

impl Endpoint {
    pub fn new(name: &str) -> Self {
        let name = trim_newline(name);

        Self {
            name,
            method: String::from("GET"),
            ..Default::default()
        }
    }

    pub fn from_state() -> Option<Self> {
        if let Ok(bytes) = crate::state::read_state() {
            if bytes.is_empty() {
                return None;
            }
            if let Ok(nesting) = String::from_utf8(bytes) {
                let nesting = nesting.split(" ").map(|s| s.to_string()).collect::<Vec<String>>();

                return match Endpoint::from_nesting(nesting) {
                    Ok(endpoint) => Some(endpoint),
                    Err(_) => None,
                };
            }
        }

        None
    }

    pub fn from_state_or_exit() -> Self {
        match Self::from_state() {
            Some(endpoint) => endpoint,
            None => {
                eprintln!("No endpoint in use. Try {}", "quartz use <ENDPOINT>".cyan());
                exit(1)
            }
        }
    }

    pub fn name_to_dir(name: &str) -> String {
        trim_newline(name.replace(&['/', '\\'], "-"))
    }

    pub fn from_dir(dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(dir.join("config.toml"))?;
        let content = String::from_utf8(bytes)?;

        let mut endpoint: Endpoint = toml::from_str(&content)?;

        endpoint.body = match std::fs::read(dir.join("body.json")) {
            Ok(bytes) => bytes.into(),
            Err(_) => Body::empty(),
        };

        Ok(endpoint)
    }

    pub fn from_nesting(mut nesting: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = Path::new(".quartz").to_path_buf();

        for parent in &nesting {
            let name = Endpoint::name_to_dir(&parent);

            path.push(name);
        }

        // Removes the actual endpoint
        nesting.pop();

        let mut endpoint = Endpoint::from_dir(path)?;
        endpoint.parents = nesting;

        Ok(endpoint)
    }

    pub fn from_name(name: &str) -> Self {
        let name = Endpoint::name_to_dir(&name);
        let dir = Path::new(".quartz").join("endpoints").join(name);

        Self::from_dir(dir).expect("Could not find endpoint")
    }

    pub fn dir(&self) -> PathBuf {
        let mut result = Path::new(".quartz").to_path_buf();

        for parent in &self.parents {
            let name = Endpoint::name_to_dir(&parent);

            result = result.join(name);
        }

        result.join(Endpoint::name_to_dir(&self.name))
    }

    pub fn nesting(&self) -> Vec<String> {
        let mut list = self.parents.clone();

        list.push(self.name.clone());

        list
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }

    pub fn parent(&self) -> Option<Endpoint> {
        let mut dir = self.dir();

        dir.pop();
        dir.pop();

        if let Ok(endpoint) = Endpoint::from_dir(dir) {
            return Some(endpoint);
        }

        None
    }

    pub fn children(&self) -> Vec<Endpoint> {
        let mut list = Vec::<Endpoint>::new();

        if let Ok(paths) = std::fs::read_dir(self.dir()) {
            for path in paths {
                let path = path.unwrap().path();

                if !path.is_dir() {
                    continue;
                }

                if let Ok(mut endpoint) = Endpoint::from_dir(path) {
                    let mut parents = self.parents.clone();
                    parents.push(self.name.clone());

                    endpoint.parents = parents;

                    list.push(endpoint);
                }
            }
        }

        list
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self) {
        let toml_content = self.to_toml().expect("Failed to generate settings.");

        std::fs::create_dir(self.dir()).expect("Failed to create endpoint.");

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.dir().join("config.toml"))
            .expect("Failed to open config file.");

        file.write(&toml_content.as_bytes())
            .expect("Failed to write to config file.");
    }

    /// Updates existing endpoint configuration file.
    // TODO: Only apply changes if a private flag is true.
    pub fn update(&self) {
        let toml_content = self.to_toml().expect("Failed to generate settings.");

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.dir().join("config.toml"))
            .expect("Failed to open config file.");

        file.write(&toml_content.as_bytes())
            .expect("Failed to write to config file.");
    }

    /// Returns the a [`Request`] based of this [`EndpointConfig`].
    pub fn as_request(&self) -> Result<Request<Body>, hyper::http::Error> {
        let mut builder = hyper::Request::builder().uri(&self.url);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        let body = match std::fs::read(self.dir().join("body.json")) {
            Ok(bytes) => bytes.into(),
            Err(_) => Body::empty(),
        };

        builder.body(body)
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
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            name: Default::default(),
            url: Default::default(),
            headers: Default::default(),
            body: Default::default(),
            parents: Default::default(),
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
