use chrono::prelude::DateTime;
use chrono::{Local, LocalResult, TimeZone, Utc};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::endpoint::{Endpoint, Headers};

pub struct History {
    unvisited: Vec<u64>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub handle: String,
    pub time: u64,
    pub request: Request,
    pub response: Response,
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
        }
    }

    pub fn from_timestemp(timestemp: u64) -> Option<Self> {
        let entry = Self {
            time: timestemp,
            ..Default::default()
        };

        if let Ok(bytes) = std::fs::read(entry.file_path()) {
            let content = String::from_utf8(bytes).unwrap();

            if let Ok(entry) = toml::from_str(&content) {
                return Some(entry);
            }
        }

        None
    }

    pub fn format_time(&self, format: &str) -> Option<String> {
        if let LocalResult::Single(utc) = Utc.timestamp_millis_opt(self.time as i64) {
            let datetime: DateTime<Local> = utc.with_timezone(&Local);
            let result = datetime.format(format).to_string();

            return Some(result);
        }

        None
    }

    pub fn file_path(&self) -> PathBuf {
        History::dir().join(self.time.to_string())
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
