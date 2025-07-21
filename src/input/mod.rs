use crate::actions::composite::{ComboAction, RepeatingAction};
use crate::actions::core::definition::create_action_from_definition;
use crate::actions::core::{ActionDefinition, Executable};
use crate::actions::{command, editing, search};
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::keymaps::KeyMap;
use crate::input::keys::{KeyEncoder, KeyEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use keys::sequence::KeySequence;

pub mod events;
pub mod keymaps;
pub mod keys;

#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub operator: Operator,
    pub repeat: Option<usize>,
}

#[derive(Debug)]
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

    fn push_operation(&mut self, operator: Operator) {
        self.pending_operation = Some(PendingOperation {
            operator,
            repeat: self.repeat,
        });
        self.repeat = None;
    }

    pub fn is_empty(&self) -> bool {
        self.repeat.is_none() && self.pending_operation.is_none() && self.sequence.is_empty()
    }

    pub fn clear(&mut self) {
        self.repeat = None;
        self.pending_operation = None;
        self.sequence.clear();
    }

    pub fn display(&self) -> String {
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
        result.push_str(&self.sequence.encode().unwrap_or_default());
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
        if let Some(ref mut repeat) = self.repeat {
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

    fn get_total_repeat(&self) -> usize {
        let motion_repeat = self.repeat.unwrap_or(1);
        let operation_repeat = self
            .pending_operation
            .as_ref()
            .and_then(|op| op.repeat)
            .unwrap_or(1);
        motion_repeat * operation_repeat
    }

    pub fn get_executable(&mut self, mode: &Mode, keymap: &KeyMap) -> Option<Box<dyn Executable>> {
        let Some(definition) = self.get_action_from_sequence(mode, keymap) else {
            return None;
        };

        if let ActionDefinition::EnterMode { mode } = &definition {
            if let Mode::OperationPending(operation) = mode {
                self.push_operation(operation.clone());
            } else {
                self.clear();
            }
            return Some(create_action_from_definition(&definition));
        }

        let repeat = self.get_total_repeat();

        if let Some(pending) = self.pending_operation.as_ref().cloned() {
            self.clear();
            if definition.is_movement_type() {
                return Some(Box::new(ComboAction::new(
                    pending.operator,
                    repeat,
                    definition,
                )));
            }
        }

        self.clear();

        let executable: Box<dyn Executable> = if repeat > 1 {
            match definition {
                ActionDefinition::DeleteCurrentLine => Box::new(ComboAction::new(
                    Operator::Delete,
                    repeat - 1,
                    ActionDefinition::MoveDown,
                )),
                ActionDefinition::ChangeCurrentLine => Box::new(ComboAction::new(
                    Operator::Change,
                    repeat - 1,
                    ActionDefinition::MoveDown,
                )),
                ActionDefinition::DeleteChar { inline } => Box::new(ComboAction::new(
                    Operator::Delete,
                    repeat,
                    ActionDefinition::MoveRight { inline },
                )),
                ActionDefinition::Backspace { inline } => Box::new(ComboAction::new(
                    Operator::Delete,
                    repeat,
                    ActionDefinition::MoveLeft { inline },
                )),
                _ => Box::new(RepeatingAction::new(repeat, definition)),
            }
        } else {
            create_action_from_definition(&definition)
        };
        Some(executable)
    }

    fn get_action_from_sequence(
        &mut self,
        mode: &Mode,
        keymap: &KeyMap,
    ) -> Option<ActionDefinition> {
        if self.sequence.keys.is_empty() {
            return None;
        };
        let action = keymap.get_action(mode, &self.sequence);

        if let Some(definition) = action {
            self.sequence.clear();
            return Some(definition.clone());
        }

        if !keymap.is_partial_match(&mode, &self.sequence) {
            self.clear();
        }
        None
    }
}

pub fn get_default_input_action(key_event: &KeyEvent, mode: &Mode) -> Option<Box<dyn Executable>> {
    let KeyEvent { code: KeyCode::Char(c), modifiers, .. } = key_event else {
        return None;
    };

    if *modifiers != KeyModifiers::NONE && *modifiers != KeyModifiers::SHIFT {
        return None;
    };

    let executable: Box<dyn Executable> = match mode {
        Mode::Insert => Box::new(editing::InsertChar::new(*c)),
        Mode::Command => Box::new(command::CommandInsertChar::new(*c)),
        Mode::Search => Box::new(search::SearchInsertChar::new(*c)),
        _ => {
            return None;
        }
    };

    Some(executable)
}
