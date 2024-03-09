use crate::{snippet, Ctx, QuartzError, QuartzResult};
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Entry {
    timestemp: i64,
    handle: String,

    /// List of exchanged HTTP messages
    messages: Vec<String>,
}

#[derive(Default)]
pub struct EntryBuilder {
    timestemp: i64,
    handle: Option<String>,
    messages: Vec<String>,
}

pub struct History {
    /// Entry timestemp identification
    entries: Vec<i64>,
}

impl History {
    pub fn new(ctx: &Ctx) -> QuartzResult<Self> {
        let paths = std::fs::read_dir(Self::dir(ctx))?;
        let mut timestemps: Vec<i64> = Vec::new();

        for path in paths {
            let timestemp = path?
                .file_name()
                .to_str()
                .ok_or(QuartzError::Internal)?
                .parse::<i64>()?;

            timestemps.push(timestemp);
        }

        timestemps.sort();
        timestemps.reverse();

        Ok(Self {
            entries: timestemps.clone(),
        })
    }

    pub fn entries(&self, ctx: &Ctx) -> Vec<Entry> {
        self.entries
            .iter()
            .filter_map(|timestemp| {
                Entry::read(&History::dir(ctx).join(timestemp.to_string())).ok()
            })
            .collect()
    }

    pub fn dir(ctx: &Ctx) -> PathBuf {
        ctx.path().join("user").join("history")
    }

    pub fn last(ctx: &Ctx) -> Option<Entry> {
        let history = History::new(ctx).ok()?;

        let entry = Entry::read(&History::dir(ctx).join(history.entries.first()?.to_string()));

        entry.ok()
    }

    pub fn write(ctx: &Ctx, entry: Entry) -> QuartzResult {
        let content = toml::to_string(&entry)?;

        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(History::dir(ctx).join(entry.timestemp.to_string()))?
            .write_all(content.as_bytes())?;

        Ok(())
    }
}

impl EntryBuilder {
    pub fn handle<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.handle = Some(value.into());
        self
    }

    pub fn message<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<snippet::Http>,
    {
        let m: snippet::Http = value.into();
        self.messages.push(m.to_string());
        self
    }

    pub fn message_raw(&mut self, value: String) -> &mut Self {
        self.messages.push(value);
        self
    }

    pub fn timestemp(&mut self, value: i64) -> &mut Self {
        self.timestemp = value;
        self
    }

    pub fn build(self) -> QuartzResult<Entry, QuartzError> {
        let handle = self.handle.ok_or(QuartzError::Internal)?;

        if self.timestemp == 0 || self.messages.is_empty() {
            return Err(QuartzError::Internal);
        }

        Ok(Entry {
            handle,
            timestemp: self.timestemp,
            messages: self.messages,
        })
    }
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn messages(&self) -> &Vec<String> {
        &self.messages
    }

    pub fn read(path: &Path) -> QuartzResult<Self> {
        let content = std::fs::read_to_string(path)?;

        Ok(toml::from_str(&content)?)
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.handle)?;
        write!(f, "{}", self.messages.join("\n"))?;

        Ok(())
    }
}
