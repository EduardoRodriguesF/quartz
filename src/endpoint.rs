use colored::Colorize;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
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
}

impl Endpoint {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            method: String::from("GET"),
            ..Default::default()
        }
    }

    pub fn from_state() -> Option<Self> {
        if let Ok(bytes) = std::fs::read(Path::new(".quartz").join("state")) {
            if bytes.is_empty() {
                return None;
            }
            if let Ok(name) = String::from_utf8(bytes) {
                return Some(Endpoint::from_name(&name));
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
        name.replace(&['/', '\\'], "-")
    }

    pub fn from_name(name: &str) -> Self {
        let name = Endpoint::name_to_dir(&name);

        let bytes = std::fs::read(
            Path::new(".quartz")
                .join("endpoints")
                .join(&name)
                .join("config.toml"),
        )
        .expect("Could not find endpoint");
        let content = String::from_utf8(bytes).unwrap();

        let mut endpoint: Endpoint = toml::from_str(&content).unwrap();

        endpoint.body = match std::fs::read(endpoint.dir().join("body.json")) {
            Ok(bytes) => bytes.into(),
            Err(_) => Body::empty(),
        };

        endpoint
    }

    pub fn dir(&self) -> PathBuf {
        let name = Endpoint::name_to_dir(&self.name);

        Path::new(".quartz").join("endpoints").join(&name)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
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
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            name: Default::default(),
            url: Default::default(),
            headers: Default::default(),
            body: Default::default(),
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
