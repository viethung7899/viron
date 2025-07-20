use crate::actions::core::ActionDefinition;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::keys::KeySequence;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct KeyMap {
    pub default: HashMap<KeySequence, ActionDefinition>,
    pub movement: HashMap<KeySequence, ActionDefinition>,
    pub normal: HashMap<KeySequence, ActionDefinition>,
    pub pending: PendingKeyMap,
}

#[derive(Debug, Default)]
pub struct PendingKeyMap {
    pub delete: HashMap<KeySequence, ActionDefinition>,
    pub change: HashMap<KeySequence, ActionDefinition>,
    pub yank: HashMap<KeySequence, ActionDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KeyMapConfig {
    #[serde(default)]
    pub default: HashMap<String, ActionDefinition>,
    #[serde(default)]
    pub movement: HashMap<String, ActionDefinition>,
    #[serde(default)]
    pub normal: HashMap<String, ActionDefinition>,
    pub pending: PendingKeyMapConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PendingKeyMapConfig {
    #[serde(default)]
    pub delete: HashMap<String, ActionDefinition>,
    #[serde(default)]
    pub change: HashMap<String, ActionDefinition>,
    #[serde(default)]
    pub yank: HashMap<String, ActionDefinition>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_action(&self, mode: &Mode, sequence: &KeySequence) -> Option<&ActionDefinition> {
        let definition = match mode {
            Mode::Normal => self
                .normal
                .get(sequence)
                .or_else(|| self.movement.get(sequence)),
            Mode::OperationPending(Operator::Delete) => self
                .movement
                .get(sequence)
                .or_else(|| self.pending.delete.get(sequence)),
            Mode::OperationPending(Operator::Change) => self
                .movement
                .get(sequence)
                .or_else(|| self.pending.change.get(sequence)),
            Mode::OperationPending(Operator::Yank) => self
                .movement
                .get(sequence)
                .or_else(|| self.pending.yank.get(sequence)),
            _ => None,
        };
        definition.or_else(|| self.default.get(sequence))
    }

    pub fn is_partial_match(&self, mode: &Mode, sequence: &KeySequence) -> bool {
        let keys: Box<dyn Iterator<Item = &KeySequence>> = match mode {
            Mode::Normal => Box::new(self.movement.keys().chain(self.normal.keys())),
            Mode::OperationPending(_) => Box::new(self.movement.keys()),
            _ => {
                return false; // No partial matches in other modes
            }
        };

        for key in keys {
            if sequence.is_prefix_of(key) && sequence.keys.len() < key.keys.len() {
                return true;
            }
        }

        false
    }

    pub fn load_from_config(config: &KeyMapConfig) -> Result<Self> {
        let mut keymap = Self::new();

        for (key_str, action_def) in &config.normal {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.normal.insert(sequence, action_def.clone());
        }

        for (key_str, action_def) in &config.default {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.default.insert(sequence, action_def.clone());
        }

        for (key_str, action_def) in &config.movement {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.movement.insert(sequence, action_def.clone());
        }

        for (key_str, action_def) in &config.pending.delete {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.pending.delete.insert(sequence, action_def.clone());
        }

        for (key_str, action_def) in &config.pending.change {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.pending.change.insert(sequence, action_def.clone());
        }
        
        for (key_str, action_def) in &config.pending.yank {
            let sequence = KeySequence::from_string(key_str)?;
            keymap.pending.yank.insert(sequence, action_def.clone());
        }

        Ok(keymap)
    }
}
