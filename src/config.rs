use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Default, Serialize)]
pub struct Config {
    pub preferences: Preferences,
    pub ui: UiConfig,
}

#[derive(Default, Deserialize)]
pub struct ConfigBuilder {
    pub preferences: PreferencesBuilder,
    pub ui: Option<UiConfig>,
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        let mut config = Config::default();
        config.ui = self.ui.unwrap_or_default();
        config.preferences = self.preferences.build();

        config
    }
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
            let config: ConfigBuilder = toml::from_str(&config_toml).unwrap();

            return config.build();
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

#[derive(Default, Debug, Deserialize)]
pub struct PreferencesBuilder {
    editor: Option<String>,
    pager: Option<String>,
}

impl PreferencesBuilder {
    pub fn editor<T>(&mut self, value: T) -> &mut PreferencesBuilder
    where
        T: Into<String>,
    {
        self.editor = Some(value.into());
        self
    }

    pub fn pager<T>(&mut self, value: T) -> &mut PreferencesBuilder
    where
        T: Into<String>,
    {
        self.pager = Some(value.into());
        self
    }

    pub fn build(self) -> Preferences {
        let mut prefs = Preferences::default();

        if let Some(editor) = self.editor {
            prefs.editor = editor;
        }

        if let Some(pager) = self.pager {
            prefs.pager = pager;
        }

        prefs
    }
}

#[derive(Serialize)]
pub struct Preferences {
    pub editor: String,
    pub pager: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            editor: "vim".to_string(),
            pager: "less".to_string(),
        }
    }
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
