use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub preferences: Preferences,
    pub ui: UiConfig,
}

impl Config {
    pub fn filename() -> String {
        ".quartz.toml".to_string()
    }

    pub fn filepath() -> PathBuf {
        let home = std::env::var("HOME").unwrap();
        let mut path = PathBuf::new();

        path.push(home);
        path.push(Config::filename());

        path
    }

    pub fn parse() -> Self {
        let filepath = Config::filepath();

        if let Ok(config_toml) = std::fs::read_to_string(filepath) {
            return toml::from_str::<Config>(&config_toml).unwrap_or_default();
        }

        Config::default()
    }

    pub fn write(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string(self)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(Self::filepath())?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

#[derive(Serialize, Default, Deserialize)]
pub struct Preferences {
    editor: Option<String>,
    pager: Option<String>,
}

impl Preferences {
    pub fn editor(&self) -> String {
        if let Some(editor) = &self.editor {
            editor.to_owned()
        } else if let Ok(editor) = std::env::var("EDITOR") {
            editor
        } else {
            "vim".to_string()
        }
    }

    pub fn set_editor<T>(&mut self, editor: T)
    where
        T: Into<String>,
    {
        self.editor = Some(editor.into());
    }

    pub fn pager(&self) -> String {
        if let Some(pager) = &self.pager {
            pager.to_owned()
        } else if let Ok(pager) = std::env::var("PAGER") {
            pager.to_string()
        } else {
            "less".to_string()
        }
    }

    pub fn set_pager<T>(&mut self, pager: T)
    where
        T: Into<String>,
    {
        self.pager = Some(pager.into());
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct UiConfig {
    colors: Option<bool>,
}

impl UiConfig {
    pub fn colors(&self) -> bool {
        if std::env::var("NO_COLOR").is_ok() {
            false
        } else if let Some(colors) = self.colors {
            colors
        } else {
            true
        }
    }

    pub fn set_colors(&mut self, colors: bool) {
        self.colors = Some(colors);
    }
}
