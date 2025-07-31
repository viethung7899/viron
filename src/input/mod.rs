use crate::actions::core::{ActionDefinition, Executable};
use crate::actions::{command, editing, search};
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::keymaps::KeyMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::actions::buffer::SetRegister;
use crate::actions::composite::{ComboAction, RepeatingAction};
use crate::actions::core::definition::create_action_from_definition;
use crate::core::register::RegisterName;
use crate::input::keys::KeyEncoder;
use crate::input::state::{InputState};
use crate::input::state::internal::RepeatState;
use crate::input::state::parser::{from_keymap_with_repeat, register, ParserResult};

pub mod events;
pub mod keymaps;
pub mod keys;
pub mod state;

#[derive(Debug)]
pub struct InputProcessor {
    state: InputState,

    // Internal states for processing input
    repeats: RepeatState,
}

impl InputProcessor {
    pub fn new() -> Self {
        InputProcessor {
            state: InputState::new(),
            repeats: RepeatState::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub fn add_key(&mut self, key_event: KeyEvent) {
        let encoded = key_event.encode().unwrap_or_default();
        log::info!("Adding key to input: {}", encoded);
        self.state.add_string(&encoded);
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.repeats.clear();
    }

    pub fn display_input(&self) -> &str {
        self.state.display()
    }

    pub fn get_executable(&mut self, mode: &Mode, keymap: &KeyMap) -> Option<Box<dyn Executable>> {
        // Get the register if it exists
        let result = register(self.state.get_input());
        match result {
            Ok((_, ParserResult { result, length })) => {
                self.state.advance(length);
                return Some(Box::new(SetRegister::new(result)));
            }
            Err(nom::Err::Incomplete(_)) => {
                // Incomplete input, wait for more input
                return None;
            }
            Err(nom::Err::Error(e)) if e.code == nom::error::ErrorKind::Eof => {
                // End of input, wait for more input
                return None;
            }
            Err(nom::Err::Error(e)) if e.code == nom::error::ErrorKind::Tag => {
                // No action for register input, continue processing
            }
            _ => {
                // Invalid register input, clear the state
                self.clear();
            }
        }

        // Get the repeat and action definition
        let result = from_keymap_with_repeat(mode, keymap)(self.state.get_input());
        match result {
            Ok((_, ParserResult { result: (optional_repeat, definition), length })) => {
                self.state.advance(length);
                self.repeats.repeat = optional_repeat;
                return Some(self.process_definition(mode, definition));
            }
            Err(nom::Err::Failure(_)) | Err(nom::Err::Error(_)) => {
                self.clear();
            }
            Err(nom::Err::Incomplete(_)) => {}
        }

        None
    }

    fn process_definition(&mut self, mode: &Mode, definition: ActionDefinition) -> Box<dyn Executable> {
        if let ActionDefinition::EnterMode { mode } = &definition {
            if matches!(mode, Mode::OperationPending(_)) {
                self.repeats.push_repeat();
            } else {
                self.clear();
            }
            return create_action_from_definition(&definition);
        }

        let repeat = self.repeats.get_total_repeat();
        if let Mode::OperationPending(operator) = mode {
            self.clear();
            if definition.is_movement_type() {
                return Box::new(ComboAction::new(*operator, repeat, definition))
            }
        }

        self.clear();

        if repeat > 1 {
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
                _ => {
                    Box::new(RepeatingAction::new(repeat, definition))
                }
            }
        } else {
            create_action_from_definition(&definition)
        }
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
