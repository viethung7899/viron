use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::editor::Mode;
use crate::input::actions::{Action, ActionDefinition, create_action_from_definition};

// Wrapper around crossterm's KeyEvent for easier handling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<KeyEvent>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn from_keys(keys: Vec<KeyEvent>) -> Self {
        Self { keys }
    }

    pub fn add(&mut self, key: KeyEvent) {
        self.keys.push(key);
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn is_prefix_of(&self, other: &KeySequence) -> bool {
        if self.keys.len() > other.keys.len() {
            return false;
        }

        for (i, key) in self.keys.iter().enumerate() {
            if *key != other.keys[i] {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Default)]
pub struct KeyMap {
    normal_mode: HashMap<KeySequence, Box<dyn Action>>,
    insert_mode: HashMap<KeySequence, Box<dyn Action>>,
    command_mode: HashMap<KeySequence, Box<dyn Action>>,
    search_mode: HashMap<KeySequence, Box<dyn Action>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KeyMapConfig {
    pub normal_mode: HashMap<String, ActionDefinition>,
    pub insert_mode: HashMap<String, ActionDefinition>,
    pub command_mode: HashMap<String, ActionDefinition>,
    pub search_mode: HashMap<String, ActionDefinition>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_action(&self, mode: &Mode, sequence: &KeySequence) -> Option<&Box<dyn Action>> {
        match mode {
            Mode::Normal => self.normal_mode.get(sequence),
            Mode::Insert => self.insert_mode.get(sequence),
            Mode::Command => self.command_mode.get(sequence),
            Mode::Search => self.search_mode.get(sequence),
        }
    }

    pub fn is_partial_match(&self, mode: &Mode, sequence: &KeySequence) -> bool {
        let map = match mode {
            Mode::Normal => &self.normal_mode,
            Mode::Insert => &self.insert_mode,
            Mode::Command => &self.command_mode,
            Mode::Search => &self.search_mode,
        };

        for key in map.keys() {
            if sequence.is_prefix_of(key) && sequence.keys.len() < key.keys.len() {
                return true;
            }
        }

        false
    }

    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let config: KeyMapConfig = toml::from_str(&content)?;

        self.load_from_config(config)
    }

    pub fn load_from_config(&mut self, config: KeyMapConfig) -> Result<()> {
        // Clear existing mappings
        self.normal_mode.clear();
        self.insert_mode.clear();
        self.command_mode.clear();
        self.search_mode.clear();

        // Load normal mode mappings
        for (key_str, action_def) in config.normal_mode {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            self.normal_mode.insert(sequence, action);
        }

        // Load insert mode mappings
        for (key_str, action_def) in config.insert_mode {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            self.insert_mode.insert(sequence, action);
        }

        // Load command mode mappings
        for (key_str, action_def) in config.command_mode {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            self.command_mode.insert(sequence, action);
        }

        // Load search mode mappings
        for (key_str, action_def) in config.search_mode {
            let sequence = KeySequence::from_string(&key_str)?;
            let action = create_action_from_definition(&action_def);
            self.search_mode.insert(sequence, action);
        }

        Ok(())
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

        for (seq, action) in &self.normal_mode {
            normal_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.insert_mode {
            insert_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.command_mode {
            command_mode.insert(seq.to_string(), action.to_serializable());
        }

        for (seq, action) in &self.search_mode {
            search_mode.insert(seq.to_string(), action.to_serializable());
        }

        KeyMapConfig {
            normal_mode,
            insert_mode,
            command_mode,
            search_mode,
        }
    }
}

impl KeySequence {
    pub fn to_string(&self) -> String {
        self.keys
            .iter()
            .map(|key| match key.code {
                KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => c.to_string(),
                KeyCode::Char(c) => format!("<{:?}-{}>", key.modifiers, c),
                _ => format!("<{:?}>", key.code),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn from_string(s: &str) -> Result<Self> {
        let mut keys = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                // Parse special key
                let mut special = String::new();
                while let Some(next) = chars.next() {
                    if next == '>' {
                        break;
                    }
                    special.push(next);
                }

                // Parse the special key
                if special.contains('-') {
                    let parts: Vec<&str> = special.split('-').collect();
                    let modifier_str = parts[0];
                    let key_str = parts[1];

                    let modifiers = match modifier_str {
                        "SHIFT" => KeyModifiers::SHIFT,
                        "CONTROL" => KeyModifiers::CONTROL,
                        "ALT" => KeyModifiers::ALT,
                        _ => KeyModifiers::NONE,
                    };

                    if key_str.len() == 1 {
                        let c = key_str.chars().next().unwrap();
                        keys.push(KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers,
                        });
                    }
                } else {
                    // Handle special keys like <Esc>, <Enter>, etc.
                    let code = match special.as_str() {
                        "Esc" => KeyCode::Esc,
                        "Enter" => KeyCode::Enter,
                        "Backspace" => KeyCode::Backspace,
                        "Tab" => KeyCode::Tab,
                        "Space" => KeyCode::Char(' '),
                        // Add more special keys as needed
                        _ => continue, // Skip unknown keys
                    };

                    keys.push(KeyEvent {
                        code,
                        modifiers: KeyModifiers::NONE,
                    });
                }
            } else {
                // Regular character
                keys.push(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                });
            }
        }

        Ok(KeySequence::from_keys(keys))
    }
}
