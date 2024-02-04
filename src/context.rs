use std::{
    collections::HashMap,
    fmt::Display,
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{Ctx, PairMap};

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

impl PairMap<'_> for Variables {
    const NAME: &'static str = "variable";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

impl Variables {
    pub fn parse(file_content: &str) -> Self {
        let mut variables = Variables::default();

        for var in file_content.split('\n').filter(|line| !line.is_empty()) {
            variables.set(var);
        }

        variables
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

    pub fn dir(&self, ctx: &Ctx) -> PathBuf {
        ctx.path().join("contexts").join(&self.name)
    }

    pub fn write(&self, ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
        let dir = self.dir(ctx);

        std::fs::create_dir(dir)?;

        self.update(ctx)?;

        Ok(())
    }

    pub fn update(&self, ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
        let mut var_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.dir(ctx).join("variables"))?;

        if !self.variables.is_empty() {
            var_file.write_all(format!("{}", self.variables.to_string()).as_bytes())?;
        }

        Ok(())
    }

    /// Returns `true` if this context already exists on the quartz project.
    pub fn exists(&self, ctx: &Ctx) -> bool {
        self.dir(ctx).exists()
    }

    pub fn parse(ctx: &Ctx, name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut context = Self::new(name);

        if let Ok(var_contents) = std::fs::read_to_string(context.dir(ctx).join("variables")) {
            context.variables = Variables::parse(&var_contents);
        }

        Ok(context)
    }
}
