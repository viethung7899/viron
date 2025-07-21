use crate::actions::core::ActionDefinition;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::keys::sequence::KeySequence;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct KeyMapItem(pub HashMap<KeySequence, ActionDefinition>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeyMap {
    default: KeyMapItem,
    movement: KeyMapItem,
    normal: KeyMapItem,
    insert: KeyMapItem,
    search: KeyMapItem,
    command: KeyMapItem,
    pending: PendingKeyMap,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PendingKeyMap {
    delete: KeyMapItem,
    change: KeyMapItem,
    yank: KeyMapItem,
}

impl KeyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_action(&self, mode: &Mode, sequence: &KeySequence) -> Option<&ActionDefinition> {
        let definition = match mode {
            Mode::Normal => self
                .normal
                .0
                .get(sequence)
                .or_else(|| self.movement.0.get(sequence)),
            Mode::Insert => self
                .insert
                .0
                .get(sequence),
            Mode::Search => self
                .search
                .0
                .get(sequence),
            Mode::Command => self
                .command
                .0
                .get(sequence),
            Mode::OperationPending(Operator::Delete) => self
                .movement
                .0
                .get(sequence)
                .or_else(|| self.pending.delete.0.get(sequence)),
            Mode::OperationPending(Operator::Change) => self
                .movement
                .0
                .get(sequence)
                .or_else(|| self.pending.change.0.get(sequence)),
            Mode::OperationPending(Operator::Yank) => self
                .movement
                .0
                .get(sequence)
                .or_else(|| self.pending.yank.0.get(sequence)),
        };
        definition.or_else(|| self.default.0.get(sequence))
    }

    pub fn is_partial_match(&self, mode: &Mode, sequence: &KeySequence) -> bool {
        let keys: Box<dyn Iterator<Item = &KeySequence>> = match mode {
            Mode::Normal => Box::new(self.movement.0.keys().chain(self.normal.0.keys())),
            Mode::OperationPending(_) => Box::new(self.movement.0.keys()),
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
}
