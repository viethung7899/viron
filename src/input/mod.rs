use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::keymaps::KeyMap;
use crate::input::keys::{KeyEvent, KeySequence};
use actions::{Action, Executable};
use crossterm::event::KeyCode;

pub mod actions;
mod command_parser;
pub mod events;
pub mod keymaps;
pub mod keys;

pub struct PendingOperation {
    pub operator: Operator,
    pub repeat: Option<usize>,
}

pub struct InputState {
    pub repeat: Option<usize>,
    pub sequence: KeySequence,
    pub pending_operation: Option<PendingOperation>,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            repeat: None,
            sequence: KeySequence::new(),
            pending_operation: None,
        }
    }

    pub fn push_operation(&mut self, operator: Operator) {
        self.pending_operation = Some(PendingOperation {
            operator,
            repeat: self.repeat,
        });
        self.repeat = None;
    }

    pub fn clear(&mut self) {
        self.repeat = None;
        self.pending_operation = None;
        self.sequence.clear();
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        if let Some(ref pending) = self.pending_operation {
            if let Some(repeat) = pending.repeat {
                result.push_str(&repeat.to_string());
            }
            result.push_str(&pending.operator.to_string());
        };
        if let Some(repeat) = self.repeat {
            result.push_str(&repeat.to_string());
        };
        result.push_str(&self.sequence.to_string());
        result
    }

    pub fn add_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_numeric() => {
                let number = c.to_digit(10).unwrap() as usize;
                self.add_number_key(number);
            }
            _ => {
                self.sequence.add(key);
            }
        }
    }

    fn add_number_key(&mut self, number: usize) {
        if let Some(ref mut pending_operation) = self.pending_operation {
            if let Some(ref mut repeat) = pending_operation.repeat {
                *repeat = *repeat * 10 + number;
            } else if number > 0 {
                pending_operation.repeat = Some(number);
            } else {
                self.clear();
            }
        } else if let Some(ref mut repeat) = self.repeat {
            *repeat = *repeat * 10 + number;
        } else if number > 0 {
            self.repeat = Some(number);
        } else {
            self.sequence.add(KeyEvent {
                code: KeyCode::Char('0'),
                modifiers: crossterm::event::KeyModifiers::NONE,
            })
        }
    }

    pub fn get_executable(&mut self, mode: &Mode, keymap: &KeyMap) -> Option<Box<dyn Executable>> {
        None
    }

    fn get_action_from_sequence(
        &mut self,
        mode: &Mode,
        keymap: &KeyMap,
    ) -> Option<Box<dyn Action>> {
        if self.sequence.keys.is_empty() {
            return None;
        };
        let action = keymap.get_action(mode, &self.sequence);

        if let Some(action) = action {
            self.clear();
            return Some(action.clone());
        };

        if !keymap.is_partial_match(&mode, &self.sequence) {
            self.clear();
        }
        None
    }
}
