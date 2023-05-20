use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::internals::layout;

pub struct Endpoint {
    pub name: String,
    pub req: hyper::Request<()>,
}

#[derive(Debug, Serialize)]
pub struct EndpointConfig {
    pub name: String,
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// List of (key, value) pairs.
    pub headers: HashMap<String, String>,
}

impl Endpoint {
    pub fn from_config<E>(config: EndpointConfig) -> Result<Self, E> {
        let mut builder = hyper::Request::builder().uri(&config.url);

        if let Ok(method) = hyper::Method::from_bytes(config.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in &config.headers {
            builder = builder.header(key, value);
        }

        match builder.body(()) {
            Ok(req) => {
                Ok(
                    Self {
                        name: config.name,
                        req,
                    }
                )
            }
            Err(_) => todo!(),
        }
    }
}

impl EndpointConfig {
    pub fn dir(&self) -> PathBuf {
        layout::which_dir().join(&self.name)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self) {
        let toml_content = self.to_toml().expect("Failed to generate settings.");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.dir().join("config.toml"))
            .expect("Failed to open config file.");

        file.write(&toml_content.as_bytes())
            .expect("Failed to write to config file.");
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
