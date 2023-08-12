use chrono::prelude::DateTime;
use chrono::{Local, LocalResult, TimeZone, Utc};
use clap::error::ErrorKind;
use colored::Colorize;
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::endpoint::{Endpoint, Headers};

pub struct History {
    unvisited: Vec<u64>,
}

pub const DEFAULT_DATE_FORMAT: &str = "%a %b %d %H:%M:%S %Y";

#[derive(Serialize, Deserialize)]
pub struct HistoryEntry {
    pub handle: String,
    pub time: u64,
    pub request: Request,
    pub response: Response,

    #[serde(skip_serializing, skip_deserializing)]
    date_format: Option<Arc<str>>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Request {
    pub body: String,
    pub endpoint: Endpoint,
    pub context: Context,
    pub duration: u64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: Headers,
    pub size: usize,
}

impl History {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let paths = std::fs::read_dir(Self::dir())?;
        let mut timestemps: Vec<u64> = Vec::new();

        for path in paths {
            let timestemp = path?.file_name().to_str().unwrap_or("").parse::<u64>()?;

            timestemps.push(timestemp);
        }

        timestemps.sort();

        Ok(Self {
            unvisited: timestemps.clone(),
        })
    }

    pub fn dir() -> PathBuf {
        Path::new(".quartz").join("user").join("history")
    }

    pub fn last() -> Option<HistoryEntry> {
        let mut history = History::new().ok()?;

        Some(history.next()?)
    }
}

impl Iterator for History {
    type Item = HistoryEntry;

    fn next(&mut self) -> Option<HistoryEntry> {
        match self.unvisited.pop() {
            Some(timestemp) => HistoryEntry::from_timestemp(timestemp),
            _ => None,
        }
    }
}

impl HistoryEntry {
    pub fn new(handle: String, request: Request, response: Response) -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            time,
            handle,
            request,
            response,
            ..Self::default()
        }
    }

    pub fn from_timestemp(timestemp: u64) -> Option<Self> {
        let entry = Self {
            time: timestemp,
            ..Self::default()
        };

        if let Ok(bytes) = std::fs::read(entry.file_path()) {
            let content = String::from_utf8(bytes).unwrap();

            if let Ok(entry) = toml::from_str(&content) {
                return Some(entry);
            }
        }

        None
    }

    pub fn date(&self) -> Option<String> {
        if let LocalResult::Single(utc) = Utc.timestamp_millis_opt(self.time as i64) {
            let format = self
                .date_format
                .clone()
                .unwrap_or(DEFAULT_DATE_FORMAT.into());

            let datetime: DateTime<Local> = utc.with_timezone(&Local);
            let result = datetime.format(&format).to_string();

            return Some(result);
        }

        None
    }

    pub fn date_format<T>(&mut self, format: T)
    where
        T: Display + Into<Arc<str>>,
    {
        self.date_format = Some(format.into());
    }

    pub fn file_path(&self) -> PathBuf {
        History::dir().join(self.time.to_string())
    }

    pub fn field_as_string(&self, key: &str) -> Result<String, clap::Error> {
        match key {
            "url" => Ok(self.request.endpoint.url.to_string()),
            "query" => Ok(self.request.endpoint.query_string()),
            "method" => Ok(self.request.endpoint.method.to_string()),
            "request.body" => Ok(self.request.body.to_string()),
            "request.headers" => Ok(self.request.endpoint.headers.to_string()),
            "status" => Ok(self.response.status.to_string()),
            "response.headers" => Ok(self.response.headers.to_string()),
            "response.size" => Ok(self.response.size.to_string()),
            "response.body" => Ok(self.response.body.to_string()),
            _ => Err(clap::Error::new(ErrorKind::InvalidValue)),
        }
    }

    /// Consumes `self` and creates a file to record it.
    pub fn write(self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string(&self)?;

        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.file_path())?
            .write_all(content.as_bytes())?;

        Ok(())
    }
}

impl Display for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Heading line
        write!(
            f,
            "{} {} -> ",
            self.request.endpoint.colored_method(),
            self.handle.yellow()
        )?;

        match hyper::StatusCode::from_u16(self.response.status) {
            Ok(status) => write!(f, "{status}")?,
            Err(..) => write!(f, "{}", self.response.status)?,
        };

        // End of heading line
        writeln!(f)?;

        // General informations
        writeln!(f, "Url: {}", self.request.endpoint.url)?;
        writeln!(f, "Context: {}", self.request.context.name)?;
        writeln!(f, "Date: {}", self.date().unwrap_or("Unknown".into()))?;

        // Body
        writeln!(f)?;
        writeln!(f, "{}", self.response.body)?;

        Ok(())
    }
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            handle: Default::default(),
            time: Default::default(),
            request: Default::default(),
            response: Default::default(),
            date_format: Default::default(),
        }
    }
}
