use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::mode::Mode;
use crate::input::actions::{create_action_from_definition, Action, ActionDefinition};
use crate::input::keys::KeySequence;

#[derive(Debug, Default)]
pub struct KeyMap {
    pub normal: HashMap<KeySequence, Box<dyn Action>>,
    pub insert: HashMap<KeySequence, Box<dyn Action>>,
    pub command: HashMap<KeySequence, Box<dyn Action>>,
    pub search: HashMap<KeySequence, Box<dyn Action>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KeyMapConfig {
    #[serde(default = "default_map")]
    pub normal: HashMap<String, ActionDefinition>,
    #[serde(default = "default_map")]
    pub insert: HashMap<String, ActionDefinition>,
    #[serde(default = "default_map")]
    pub command: HashMap<String, ActionDefinition>,
    #[serde(default = "default_map")]
    pub search: HashMap<String, ActionDefinition>,
}

fn default_map() -> HashMap<String, ActionDefinition> {
    HashMap::new()
}

impl KeyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_action(&self, mode: &Mode, sequence: &KeySequence) -> Option<&Box<dyn Action>> {
        match mode {
            Mode::Normal => self.normal.get(sequence),
            Mode::Insert => self.insert.get(sequence),
            Mode::Command => self.command.get(sequence),
            Mode::Search => self.search.get(sequence),
        }
    }

    pub fn is_partial_match(&self, mode: &Mode, sequence: &KeySequence) -> bool {
        let map = match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
            Mode::Command => &self.command,
            Mode::Search => &self.search,
        };

        for key in map.keys() {
            if sequence.is_prefix_of(key) && sequence.keys.len() < key.keys.len() {
                return true;
            }
        }

        false
    }

    pub fn load_from_config(config: &KeyMapConfig) -> Result<Self> {
        let mut keymap = Self::new();

        // Load normal mode mappings
        for (key_str, action_def) in &config.normal {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            keymap.normal.insert(sequence, action);
        }

        // Load insert mode mappings
        for (key_str, action_def) in &config.insert {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            keymap.insert.insert(sequence, action);
        }

        // Load command mode mappings
        for (key_str, action_def) in &config.command {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            keymap.command.insert(sequence, action);
        }

        // Load search mode mappings
        for (key_str, action_def) in &config.search {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            keymap.search.insert(sequence, action);
        }

        Ok(keymap)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let config = self.to_config();
        let toml_str = toml::to_string_pretty(&config)?;
        fs::write(path, toml_str)?;
        Ok(())
    }

    pub fn to_config(&self) -> KeyMapConfig {
        let mut normal_mode = HashMap::new();
        let mut insert_mode = HashMap::new();
        let mut command_mode = HashMap::new();
        let mut search_mode = HashMap::new();

        for (seq, action) in &self.normal {
            normal_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.insert {
            insert_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.command {
            command_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.search {
            search_mode.insert(seq.to_string(), action.to_serializable());
        }

        KeyMapConfig {
            normal: normal_mode,
            insert: insert_mode,
            command: command_mode,
            search: search_mode,
        }
    }
}
