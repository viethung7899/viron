pub mod editor;

use crate::config::editor::Gutter;
use crate::input::keymaps::{KeyMap};
use crate::ui::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_DIRECTORY: &str = ".viron";

#[derive(Serialize, Deserialize)]
pub struct FileConfig {
    pub theme: String,
    #[serde(default)]
    pub gutter: Gutter,
    pub keymap: KeyMap,
}

impl FileConfig {
    fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let string = std::fs::read_to_string(path)?;
        let config = toml::from_str(&string)?;
        Ok(config)
    }
}

pub fn get_config_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_default();
    home_dir.join(CONFIG_DIRECTORY)
}

#[derive(Debug, Default)]
pub struct Config {
    pub theme: Theme,
    pub gutter: Gutter,
    pub keymap: KeyMap,
}

impl TryFrom<FileConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(file_config: FileConfig) -> Result<Self, Self::Error> {
        let theme_path = get_config_dir().join(format!("themes/{}.json", file_config.theme));
        let theme = Theme::load_from_file(&theme_path)?;
        Ok(Self {
            theme,
            keymap: file_config.keymap,
            gutter: file_config.gutter,
        })
    }
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_config = FileConfig::load_from_file(path)?;
        let config = Config::try_from(file_config)?;
        Ok(config)
    }
}
