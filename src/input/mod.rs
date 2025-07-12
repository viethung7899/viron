use crate::core::mode;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::actions::{
    create_action_from_definition, ActionDefinition, ComboAction, RepeatingAction,
};
use crate::input::keymaps::KeyMap;
use crate::input::keys::{KeyEvent, KeySequence};
use actions::{Action, Executable};
use crossterm::event::{KeyCode, KeyModifiers};

pub mod actions;
mod command_parser;
pub mod events;
pub mod keymaps;
pub mod keys;

#[derive(Debug)]
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
        let action = create_action_from_definition(&definition);

        match &definition {
            ActionDefinition::EnterPendingOperation { operator } => {
                self.push_operation(*operator);
                return Some(action);
            }
            _ => {}
        };

        let repeat = self.get_total_repeat();

        let executable: Box<dyn Executable> = if let Some(pending) = self.pending_operation.as_ref()
        {
            Box::new(ComboAction::new(pending.operator, repeat, action))
        } else if repeat > 1 {
            Box::new(RepeatingAction::new(repeat, action))
        } else {
            action
        };
        self.clear();
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

pub fn get_default_insert_action(key_event: &KeyEvent) -> Option<Box<dyn Executable>> {
    let executable: Box<dyn Executable> = match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => Box::new(actions::EnterMode::new(mode::Mode::Normal)),
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            Box::new(actions::InsertChar::new(ch))
        }
        (KeyCode::Backspace, _) => Box::new(actions::Backspace),
        (KeyCode::Delete, _) => Box::new(actions::DeleteChar),
        (KeyCode::Left, _) => Box::new(actions::MoveLeft::new(true)),
        (KeyCode::Right, _) => Box::new(actions::MoveRight::new(true)),
        (KeyCode::Up, _) => Box::new(actions::MoveUp),
        (KeyCode::Down, _) => Box::new(actions::MoveDown),
        (KeyCode::Enter, _) => Box::new(actions::InsertNewLine),
        (KeyCode::Home, _) => Box::new(actions::MoveToLineStart),
        (KeyCode::End, _) => Box::new(actions::MoveToLineEnd),
        _ => return None,
    };
    Some(executable)
}

pub fn get_default_command_action(key_event: &KeyEvent) -> Option<Box<dyn Executable>> {
    let executable: Box<dyn Executable> = match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => Box::new(actions::EnterMode::new(mode::Mode::Normal)),
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            Box::new(actions::CommandInsertChar::new(ch))
        }
        (KeyCode::Enter, _) => Box::new(actions::CommandExecute),
        (KeyCode::Left, _) => Box::new(actions::CommandMoveLeft),
        (KeyCode::Right, _) => Box::new(actions::CommandMoveLeft),
        (KeyCode::Backspace, _) => Box::new(actions::CommandBackspace),
        (KeyCode::Delete, _) => Box::new(actions::CommandDeleteChar),
        _ => {
            return None;
        }
    };
    Some(executable)
}

pub fn get_default_search_action(key_event: &KeyEvent) -> Option<Box<dyn Executable>> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            Some(Box::new(actions::SearchInsertChar::new(ch)))
        }
        (KeyCode::Enter, _) => Some(Box::new(actions::SearchSubmit)),
        (KeyCode::Left, _) => Some(Box::new(actions::SearchMoveLeft)),
        (KeyCode::Right, _) => Some(Box::new(actions::SearchMoveLeft)),
        (KeyCode::Backspace, _) => Some(Box::new(actions::SearchBackspace)),
        (KeyCode::Delete, _) => Some(Box::new(actions::SearchDeleteChar)),
        _ => None,
    }
}
