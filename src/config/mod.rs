use crate::input::keymaps::KeyMapConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_DIRECTORY: &str = ".viron";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub keymap: KeyMapConfig,
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let string = std::fs::read_to_string(path)?;
        let config = toml::from_str(&string)?;
        Ok(config)
    }
}

pub fn get_config_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_default();
    home_dir.join(CONFIG_DIRECTORY)
}
