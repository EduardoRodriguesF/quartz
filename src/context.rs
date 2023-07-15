use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            variables: HashMap::default(),
        }
    }
}

impl Context {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn dir(&self) -> PathBuf {
        Path::new(".quartz").join("contexts").join(&self.name)
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = self.dir();

        std::fs::create_dir(dir)?;

        self.update()?;

        Ok(())
    }

    pub fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::ser::to_string(&self.variables)?;

        let mut var_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.dir().join("variables.toml"))?;

        if !content.is_empty() {
            var_file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    /// Returns `true` if this context already exists on the quartz project.
    pub fn exists(&self) -> bool {
        self.dir().exists()
    }

    pub fn parse(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut context = Self::new(name);

        let var_contents = std::fs::read_to_string(context.dir().join("variables.toml"))?;

        if let Ok(variables) = toml::de::from_str(&var_contents) {
            context.variables = variables;
        } else {
            eprintln!("Malformed variables file");
        }

        Ok(context)
    }
}
