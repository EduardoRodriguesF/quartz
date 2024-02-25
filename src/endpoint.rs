use colored::Colorize;
use hyper::http::uri::InvalidUri;
use hyper::{Body, Request, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::env::{Env, Variables};
use crate::state::StateField;
use crate::{Ctx, PairMap};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Query(pub HashMap<String, String>);

impl Deref for Query {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}={value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Query {
    const NAME: &'static str = "query param";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Headers(pub HashMap<String, String>);

impl Deref for Headers {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}: {value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Headers {
    const NAME: &'static str = "header";
    const EXPECTED: &'static str = "<key>: [value]";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }

    fn pair(input: &str) -> Option<(String, String)> {
        let (key, value) = input.split_once(": ")?;

        Some((key.to_string(), value.to_string()))
    }
}

#[derive(Debug)]
pub struct EndpointHandle {
    /// List of ordered parent names
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// Query params.
    pub query: Query,

    /// List of (key, value) pairs.
    pub headers: Headers,

    /// Variable values applied from a [`Env`]
    #[serde(skip_serializing, skip_deserializing)]
    pub variables: Variables,

    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
}

#[derive(Default, Debug, clap::Args)]
pub struct EndpointPatch {
    /// Request URL
    #[arg(long)]
    pub url: Option<String>,

    /// HTTP request method
    #[arg(short = 'X', long = "request")]
    pub method: Option<String>,

    /// Add a parameter the URL query
    #[arg(short, long, value_name = "PARAM")]
    pub query: Vec<String>,

    /// Add a header entry in "<key>: <value>" format. This argument can be passed multiple times
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,
}

impl EndpointPatch {
    pub fn has_changes(&self) -> bool {
        self.url.is_some()
            || self.method.is_some()
            || !self.query.is_empty()
            || !self.headers.is_empty()
    }
}

impl<T> From<T> for EndpointHandle
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let path: Vec<String> = value.as_ref().split('/').map(|s| s.to_string()).collect();

        Self { path }
    }
}

impl EndpointHandle {
    /// Points to top-level quartz folder.
    ///
    /// This constant can be used to traverse through all handles starting
    /// from the top one.
    pub const QUARTZ: Self = Self { path: vec![] };

    pub fn from_state(ctx: &Ctx) -> Option<Self> {
        if let Ok(handle) = ctx.state.get(ctx, StateField::Endpoint) {
            if handle.is_empty() {
                return None;
            }

            return Some(EndpointHandle::from(handle));
        }

        None
    }

    pub fn head(&self) -> String {
        self.path.last().unwrap_or(&String::new()).clone()
    }

    pub fn dir(&self, ctx: &Ctx) -> PathBuf {
        let mut result = ctx.path().join("endpoints");

        for parent in &self.path {
            let name = Endpoint::name_to_dir(parent);

            result = result.join(name);
        }

        result
    }

    pub fn handle(&self) -> String {
        self.path.join("/")
    }

    pub fn exists(&self, ctx: &Ctx) -> bool {
        self.dir(ctx).exists()
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self, ctx: &Ctx) {
        let mut dir = ctx.path().join("endpoints");
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

        std::fs::create_dir_all(self.dir(ctx))
            .unwrap_or_else(|_| panic!("failed to create endpoint"));
    }

    /// Removes endpoint to make it an empty handle
    pub fn make_empty(&self, ctx: &Ctx) {
        if let Some(_) = self.endpoint(ctx) {
            let _ = std::fs::remove_file(self.dir(ctx).join("endpoint.toml"));
            let _ = std::fs::remove_file(self.dir(ctx).join("body"));
        }
    }

    pub fn children(&self, ctx: &Ctx) -> Vec<EndpointHandle> {
        let mut list = Vec::<EndpointHandle>::new();

        if let Ok(paths) = std::fs::read_dir(self.dir(ctx)) {
            for path in paths {
                let path = path.unwrap().path();

                if !path.is_dir() {
                    continue;
                }

                if let Ok(vec) = std::fs::read(path.join("spec")) {
                    let spec = String::from_utf8(vec).unwrap_or_else(|_| {
                        panic!("failed to get handle");
                    });

                    let mut path = self.path.clone();
                    path.push(spec);

                    list.push(EndpointHandle { path })
                }
            }
        }

        list
    }

    #[must_use]
    pub fn endpoint(&self, ctx: &Ctx) -> Option<Endpoint> {
        Endpoint::from_dir(&self.dir(ctx)).ok()
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        let handle = self.handle().replace(from, to);
        self.path = EndpointHandle::from(handle).path;
    }
}

impl From<&mut EndpointPatch> for Endpoint {
    fn from(value: &mut EndpointPatch) -> Self {
        let mut endpoint = Self::default();
        endpoint.update(value);

        endpoint
    }
}

impl Endpoint {
    pub fn new(path: PathBuf) -> Self {
        Self {
            method: String::from("GET"),
            path,
            ..Default::default()
        }
    }

    pub fn name_to_dir(name: &str) -> String {
        name.trim().replace(['/', '\\'], "-")
    }

    pub fn from_dir(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(dir.join("endpoint.toml"))?;
        let content = String::from_utf8(bytes)?;

        let mut endpoint: Endpoint = toml::from_str(&content)?;
        endpoint.path = dir.to_path_buf();

        Ok(endpoint)
    }

    pub fn update(&mut self, src: &mut EndpointPatch) {
        if let Some(method) = &mut src.method {
            std::mem::swap(&mut self.method, method);
        }

        if let Some(url) = &mut src.url {
            std::mem::swap(&mut self.url, url);
        }

        for input in &src.query {
            self.query.set(input);
        }

        for input in &src.headers {
            self.headers.set(input);
        }

        for input in &src.query {
            self.query.set(input);
        }
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }

    pub fn has_body(&self) -> bool {
        self.path.join("body").exists()
    }

    pub fn body(&self) -> String {
        match std::fs::read_to_string(self.path.join("body")) {
            Ok(mut content) => {
                for (key, value) in self.variables.iter() {
                    let key_match = format!("{{{{{}}}}}", key);

                    content = content.replace(&key_match, value);
                }

                content.into()
            }
            Err(_) => "".to_string(),
        }
    }

    pub fn set_handle(&mut self, ctx: &Ctx, handle: &EndpointHandle) {
        self.path = handle.dir(ctx).to_path_buf();
    }

    pub fn apply_env(&mut self, env: &Env) {
        for (key, value) in env.variables.iter() {
            let key_match = format!("{{{{{}}}}}", key); // {{key}}

            self.url = self.url.replace(&key_match, value);
            self.method = self.method.replace(&key_match, value);

            *self.headers = self
                .headers
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();

            *self.query = self
                .query
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();
        }

        self.variables = env.variables.clone();
    }

    pub fn full_url(&self) -> Result<Uri, InvalidUri> {
        let query_string = self.query_string();

        let mut url = self.url.clone();

        if !query_string.is_empty() {
            let delimiter = if self.url.contains('?') { '&' } else { '?' };
            url.push(delimiter);
            url.push_str(&query_string);
        }

        let result = Uri::try_from(&url);

        if result.is_err() {
            if !url.contains("://") {
                let mut scheme = "http://".to_owned();
                scheme.push_str(&url);

                return Uri::try_from(scheme);
            }
        }

        result
    }

    /// Returns the a [`Request`] consuming struct.
    pub fn into_request<T>(self, body: T) -> Result<Request<Body>, hyper::http::Error>
    where
        T: Into<Body>,
    {
        let mut builder = hyper::Request::builder().uri(&self.full_url()?);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        builder.body(body.into())
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
    /// would return: v=9000&fields=lorem,ipsum
    pub fn query_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in self.query.iter() {
            result.push(format!("{key}={value}"));
        }

        result.sort();
        result.join("&")
    }

    pub fn write(&mut self) {
        let toml_content = self
            .to_toml()
            .unwrap_or_else(|_| panic!("failed to generate settings"));

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path.join("endpoint.toml"))
            .unwrap_or_else(|_| panic!("failed to open config file"));

        file.write_all(toml_content.as_bytes())
            .unwrap_or_else(|_| panic!("failed to write to config file"));
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
            path: Default::default(),
        }
    }
}
