use std::{
    collections::HashMap,
    fmt::Display,
    io::Write,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Variables(pub HashMap<String, String>);

impl Deref for Variables {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}={value}")?;
        }

        Ok(())
    }
}

impl Variables {
    /// Sets a variable key-value pair based on a var string.
    ///
    /// Expects <key>=<value> string.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::context::Variables;
    /// let mut variables = Variables::default();
    /// variables.set("myvar=\"some_value\"");
    ///
    /// assert_eq!(variables.get("myvar").unwrap(), "some_value");
    /// ```
    ///
    /// # Panics
    ///
    /// Passing an invalid variable string.
    ///
    /// ```should_panic
    /// use quartz_cli::context::Variables;
    /// let mut variables = Variables::default();
    ///
    /// variables.set("invalid_variable");
    /// ```
    pub fn set(&mut self, variable: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (key, value) = variable
            .split_once('=')
            .expect("malformed variable. Expected <key>=<value>");

        let value = value.trim_matches('\'').trim_matches('\"');

        self.insert(key.to_string(), value.to_string());

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
    pub name: String,
    pub variables: Variables,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            variables: Variables::default(),
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
            panic!("malformed variables file");
        }

        Ok(context)
    }
}
