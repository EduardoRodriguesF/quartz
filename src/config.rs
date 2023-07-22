use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Default, Deserialize, Serialize)]
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
            let config: Config = toml::from_str(&config_toml).unwrap();

            return config;
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

#[derive(Serialize, Deserialize)]
pub struct Preferences {
    pub editor: String,
}

#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    pub colors: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { colors: true }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            editor: "vim".to_string(),
        }
    }
}
