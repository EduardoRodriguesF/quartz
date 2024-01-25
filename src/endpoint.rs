use colored::Colorize;
use hyper::http::uri::InvalidUri;
use hyper::{Body, Request, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::context::{Context, Variables};
use crate::state::{State, StateField};

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

impl Query {
    /// Sets a query param.
    ///
    /// Expects "<key>=<value>" format.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::endpoint::Query;
    /// let mut query = Query::default();
    /// query.set("someparam=value");
    ///
    /// assert_eq!(query.get("someparam").unwrap(), "value");
    /// ```
    ///
    /// # Panics
    ///
    /// Passing an invalid header string.
    ///
    /// ```should_panic
    /// use quartz_cli::endpoint::Query;
    /// let mut query = Query::default();
    ///
    /// query.set("invalid: value");
    /// ```
    pub fn set(&mut self, param: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (key, value) = param
            .split_once('=')
            .expect("malformed query param. Expected <key>=<value>");

        self.insert(key.to_string(), value.to_string());

        Ok(())
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

impl Headers {
    /// Sets a header.
    ///
    /// Expects "<key>: <value>" format.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::endpoint::Headers;
    /// let mut headers = Headers::default();
    /// headers.set("content-type: application/json");
    ///
    /// assert_eq!(headers.get("content-type").unwrap(), "application/json");
    /// ```
    ///
    /// # Panics
    ///
    /// Passing an invalid header string.
    ///
    /// ```should_panic
    /// use quartz_cli::endpoint::Headers;
    /// let mut headers = Headers::default();
    ///
    /// headers.set("invalid=value");
    /// ```
    pub fn set(&mut self, header: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (key, value) = header
            .split_once(": ")
            .expect("malformed variable: {}\nexpected <key>=<value>");

        self.insert(key.to_string(), value.to_string());

        Ok(())
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

    /// Variable values applied from a [`Context`]
    #[serde(skip_serializing, skip_deserializing)]
    pub variables: Variables,
}

impl EndpointHandle {
    /// Points to top-level quartz folder.
    ///
    /// This constant can be used to traverse through all specifications starting
    /// from the top one.
    pub const QUARTZ: Self = Self { path: vec![] };

    pub fn from_handle<S>(handle: S) -> Self
    where
        S: AsRef<str>,
    {
        let path: Vec<String> = handle.as_ref().split('/').map(|s| s.to_string()).collect();

        Self { path }
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
        self.path.join("/")
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

        std::fs::create_dir_all(self.dir()).unwrap_or_else(|_| panic!("failed to create endpoint"));
    }

    pub fn children(&self) -> Vec<EndpointHandle> {
        let mut list = Vec::<EndpointHandle>::new();

        if let Ok(paths) = std::fs::read_dir(self.dir()) {
            for path in paths {
                let path = path.unwrap().path();

                if !path.is_dir() {
                    continue;
                }

                if let Ok(vec) = std::fs::read(path.join("spec")) {
                    let spec = String::from_utf8(vec).unwrap_or_else(|_| {
                        panic!("failed to get endpoint specification");
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
    pub fn endpoint(&self) -> Option<Endpoint> {
        Endpoint::from_dir(self.dir()).ok()
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

                for (key, value) in self.variables.iter() {
                    let key_match = format!("{{{{{}}}}}", key);

                    content = content.replace(&key_match, value);
                }

                content.into()
            }
            Err(_) => Body::empty(),
        }
    }

    pub fn apply_context(&mut self, context: &Context) {
        for (key, value) in context.variables.iter() {
            let key_match = format!("{{{{{}}}}}", key);

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

        self.variables = context.variables.clone();
    }

    pub fn full_url(&self) -> Result<Uri, InvalidUri> {
        let query_string = self.query_string();

        let mut url = self.url.clone();

        if !query_string.is_empty() {
            let delimiter = if self.url.contains('?') { '&' } else { '?' };
            url.push(delimiter);
            url.push_str(&query_string);
        }

        Uri::try_from(url)
    }

    /// Returns the a [`Request`] consuming struct.
    pub fn into_request(self, spec: &EndpointHandle) -> Result<Request<Body>, hyper::http::Error> {
        let mut builder = hyper::Request::builder().uri(&self.full_url()?);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in self.headers.iter() {
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
    /// would return: v=9000&fields=lorem,ipsum
    pub fn query_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in self.query.iter() {
            result.push(format!("{key}={value}"));
        }

        result.sort();
        result.join("&")
    }

    pub fn write(&mut self, handle: EndpointHandle) {
        let toml_content = self
            .to_toml()
            .unwrap_or_else(|_| panic!("failed to generate settings"));

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(handle.dir().join("endpoint.toml"))
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
