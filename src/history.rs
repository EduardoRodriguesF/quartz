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
    path: Vec<String>,
    endpoint: Option<Endpoint>,
    context: Option<Context>,
    time: u64,
    duration: u64,
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

    pub fn dir() -> PathBuf {
        Path::new(".quartz").join("user").join("history")
    }
}

impl RequestHistoryEntry {
    pub fn new() -> Self {
        let mut entry = Self::default();

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        entry.time = time as u64;

        entry
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
            .write(content.as_bytes())?;

        Ok(())
    }
}
