use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

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
    pub fn dir(&self) -> PathBuf {
        Path::new(".quartz").join("contexts").join(&self.name)
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = self.dir();

        std::fs::create_dir(&dir)?;

        self.update()?;

        Ok(())
    }

    pub fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::ser::to_string(&self.variables)?;

        let mut var_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.dir().join("variables.toml"))?;

        if !content.is_empty() {
            var_file.write(content.as_bytes())?;
        }

        Ok(())
    }
}
