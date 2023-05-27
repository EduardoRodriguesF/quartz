use hyper::{Client, Body};
use hyper::http::{Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::internals::layout;

pub struct Endpoint {
    pub name: String,
    pub req: hyper::Request<Body>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// List of (key, value) pairs.
    pub headers: HashMap<String, String>,
}

impl Endpoint {
    pub fn from_config(config: EndpointConfig) -> Result<Self, Box<dyn Error>> {
        let mut builder = hyper::Request::builder().uri(&config.url);

        if let Ok(method) = hyper::Method::from_bytes(config.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in &config.headers {
            builder = builder.header(key, value);
        }

        let body = match std::fs::read(config.dir().join("body.json")) {
            Ok(bytes) => bytes.into(),
            Err(_) => Body::empty(),
        };

        match builder.body(body) {
            Ok(req) => Ok(Self {
                name: config.name,
                req,
            }),
            Err(_) => todo!(),
        }
    }

    pub async fn send(&self) -> Result<Response<Body>, hyper::Error> {
        let client = Client::new();

        client.request(self.clone().req).await
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        let mut builder = Request::builder()
            .uri(self.req.uri())
            .method(self.req.method());

        for (key, value) in self.req.headers() {
            builder = builder.header(key, value);
        }

        let req = builder.body(Body::empty()).unwrap();

        Self {
            name: self.name.clone(),
            req,
        }
    }
}

impl EndpointConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            method: String::from("GET"),
            ..Default::default()
        }
    }

    pub fn from_name(name: &str) -> Self {
        let bytes = std::fs::read(layout::which_dir().join(&name).join("config.toml"))
            .expect("Could not find endpoint");
        let content = String::from_utf8(bytes).unwrap();

        toml::from_str(&content).unwrap()
    }

    pub fn dir(&self) -> PathBuf {
        layout::which_dir().join(&self.name)
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
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            name: Default::default(),
            url: Default::default(),
            headers: Default::default(),
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
