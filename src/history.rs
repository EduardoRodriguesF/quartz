use chrono::prelude::DateTime;
use chrono::{Local, LocalResult, TimeZone, Utc};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::endpoint::Endpoint;

pub struct RequestHistory {
    timestemps: Vec<u64>,
    unvisited: Vec<u64>,
    requests: HashMap<u64, RequestHistoryEntry>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct RequestHistoryEntry {
    pub path: Vec<String>,
    pub endpoint: Option<Endpoint>,
    pub context: Option<Context>,
    pub time: u64,
    pub duration: u64,
    pub status: Option<u16>,
    pub body: String,
}

impl RequestHistory {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let paths = std::fs::read_dir(Self::dir())?;
        let mut timestemps: Vec<u64> = Vec::new();

        for path in paths {
            let timestemp = path?.file_name().to_str().unwrap_or("").parse::<u64>()?;

            timestemps.push(timestemp);
        }

        timestemps.sort();

        Ok(Self {
            requests: HashMap::default(),
            unvisited: timestemps.clone(),
            timestemps,
        })
    }

    pub fn next(&mut self) -> Option<RequestHistoryEntry> {
        match self.unvisited.pop() {
            Some(timestemp) => RequestHistoryEntry::from_timestemp(timestemp),
            _ => None,
        }
    }

    pub fn dir() -> PathBuf {
        Path::new(".quartz").join("user").join("history")
    }
}

impl RequestHistoryEntry {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            time,
            ..Default::default()
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

    pub fn path(&mut self, path: Vec<String>) -> &mut Self {
        self.path = path;

        self
    }

    pub fn endpoint(&mut self, endpoint: &Endpoint) -> &mut Self {
        self.endpoint = Some(endpoint.clone());

        self
    }

    pub fn context(&mut self, context: &Context) -> &mut Self {
        self.context = Some(context.clone());

        self
    }

    pub fn duration(&mut self, duration: u64) -> &mut Self {
        self.duration = duration;

        self
    }

    pub fn status(&mut self, status: u16) -> &mut Self {
        self.status = Some(status);

        self
    }

    pub fn body(&mut self, bytes: &[u8]) -> &mut Self {
        self.body = String::from_utf8_lossy(bytes).to_string();

        self
    }

    pub fn file_path(&self) -> PathBuf {
        RequestHistory::dir().join(self.time.to_string())
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
