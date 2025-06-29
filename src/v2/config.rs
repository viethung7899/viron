use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::input::keymaps::KeyMapConfig;

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
