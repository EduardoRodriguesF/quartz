use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub enum State {
    Endpoint,
    Context,
}

impl State {
    pub const STATE_DIR: &str = ".quartz/user/state";

    pub fn file_path(&self) -> PathBuf {
        Path::new(Self::STATE_DIR).join(match self {
            Self::Endpoint => "endpoint",
            Self::Context => "context",
        })
    }

    pub fn get(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(self.file_path())?;

        Ok(String::from_utf8(bytes)?)
    }

    pub fn set(&self, value: &str) -> Result<(), std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(self.file_path());

        file?.write_all(value.as_bytes())
    }
}
