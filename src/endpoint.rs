use colored::Colorize;
use hyper::{Body, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug)]
pub struct Specification {
    pub endpoint: Option<Endpoint>,

    /// List of ordered parent names
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// List of (key, value) pairs.
    pub headers: HashMap<String, String>,

    #[serde(skip_serializing, skip_deserializing)]
    pub body: Body,
}

impl Specification {
    /// Points to top-level quartz folder.
    ///
    /// This constant can be used to traverse through all specifications starting
    /// from the top one.
    pub const QUARTZ: Self = Self {
        path: vec![],
        endpoint: None,
    };

    pub fn from_nesting(nesting: Vec<String>) -> Self {
        let mut path = Path::new(".quartz").to_path_buf();

        for parent in &nesting {
            let name = Endpoint::name_to_dir(&parent);

            path.push(name);
        }

        let endpoint = match Endpoint::from_dir(path) {
            Ok(endpoint) => Some(endpoint),
            Err(_) => None,
        };

        Self {
            path: nesting,
            endpoint,
        }
    }

    pub fn from_state() -> Option<Self> {
        if let Ok(bytes) = crate::state::read_state() {
            if bytes.is_empty() {
                return None;
            }

            if let Ok(nesting) = String::from_utf8(bytes) {
                let nesting = nesting
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                return Some(Specification::from_nesting(nesting));
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

    pub fn head(&self) -> String {
        self.path.last().unwrap_or(&String::new()).clone()
    }

    pub fn dir(&self) -> PathBuf {
        let mut result = Path::new(".quartz").join("endpoints");

        for parent in &self.path {
            let name = Endpoint::name_to_dir(&parent);

            result = result.join(name);
        }

        result
    }

    pub fn nesting(&self) -> Vec<String> {
        let mut list = self.path.clone();

        list.push(self.head());

        list
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self) {
        std::fs::create_dir_all(self.dir()).expect("Failed to create endpoint.");

        if let Some(endpoint) = &self.endpoint {
            let toml_content = endpoint.to_toml().expect("Failed to generate settings.");

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(self.dir().join("endpoint.toml"))
                .expect("Failed to open config file.");

            file.write(&toml_content.as_bytes())
                .expect("Failed to write to config file.");
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.dir().join("spec"))
            .unwrap();

        let _ = file.write(self.head().as_bytes());
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

            file.write(&toml_content.as_bytes())
                .expect("Failed to write to config file.");
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.dir().join("spec"))
            .unwrap();

        let _ = file.write(self.head().as_bytes());
    }

    pub fn children(&self) -> Vec<Specification> {
        let mut list = Vec::<Specification>::new();

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
                        eprintln!("Failed to get endpoint specification");
                        exit(1);
                    });

                    let mut path = self.path.clone();
                    path.push(spec);

                    list.push(Specification {
                        path,
                        endpoint,
                    })
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
        trim_newline(name.replace(&['/', '\\'], "-"))
    }

    pub fn from_dir(dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(dir.join("endpoint.toml"))?;
        let content = String::from_utf8(bytes)?;

        let mut endpoint: Endpoint = toml::from_str(&content)?;

        endpoint.body = match std::fs::read(dir.join("body.json")) {
            Ok(bytes) => bytes.into(),
            Err(_) => Body::empty(),
        };

        Ok(endpoint)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
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

        builder.body(self.clone().body)
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
            url: Default::default(),
            headers: Default::default(),
            body: Default::default(),
        }
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        let body = self.as_request().unwrap().into_body();

        Self {
            url: self.url.clone(),
            method: self.method.clone(),
            headers: self.headers.clone(),
            body,
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
