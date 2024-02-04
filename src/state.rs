use crate::Ctx;
use std::{io::Write, path::PathBuf};

pub enum StateField {
    Endpoint,
    Env,
}

pub struct State {
    pub handle: Option<String>,
}

impl StateField {
    pub const STATE_DIR: &str = "user/state";

    pub fn file_path(&self, ctx: &Ctx) -> PathBuf {
        ctx.path().join(Self::STATE_DIR).join(match self {
            Self::Endpoint => "endpoint",
            Self::Env => "env",
        })
    }

    pub fn get(&self, ctx: &Ctx) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(self.file_path(ctx))?;

        Ok(String::from_utf8(bytes)?)
    }

    pub fn set(&self, ctx: &Ctx, value: &str) -> Result<(), std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(self.file_path(ctx));

        file?.write_all(value.as_bytes())
    }
}

impl State {
    pub fn get(&self, ctx: &Ctx, field: StateField) -> Result<String, Box<dyn std::error::Error>> {
        let overwrite = match field {
            StateField::Endpoint => self.handle.clone(),
            _ => None,
        };

        if let Some(overwrite) = overwrite {
            return Ok(overwrite);
        }

        field.get(ctx)
    }
}
