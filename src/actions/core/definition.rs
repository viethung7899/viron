use crate::actions::core::{Action, CompositeAction};
use crate::actions::types::{buffer, editing, lsp, mode, movement, search, system};
use crate::core::mode::Mode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum ActionDefinition {
    // Movement actions
    MoveLeft {
        inline: bool,
    },
    MoveRight {
        inline: bool,
    },
    MoveUp,
    MoveDown,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToTop,
    MoveToBottom,
    MoveToViewportCenter,
    MoveToPreviousWord,
    MoveToNextWord,
    GoToLine {
        line_number: usize,
    },

    // Editing actions
    InsertChar {
        ch: char,
    },
    InsertNewLine,
    InsertNewLineBelow,
    InsertNewLineAbove,

    Backspace {
        inline: bool,
    },
    DeleteChar {
        inline: bool,
    },
    DeleteCurrentLine,
    ChangeCurrentLine,

    Undo,
    Redo,

    // Search actions
    FindNext,
    FindPrevious,

    // Mode actions
    EnterMode {
        mode: Mode,
    },

    // Buffer actions
    NextBuffer,
    PreviousBuffer,
    OpenBuffer {
        path: String,
    },
    WriteBuffer {
        path: Option<String>,
    },
    CloseBuffer {
        force: bool,
    },

    // LSP actions
    GoToDefinition,

    // System actions
    Quit,

    // Composite actions
    Composite {
        description: String,
        actions: Vec<ActionDefinition>,
    },
}

pub fn create_action_from_definition(definition: &ActionDefinition) -> Box<dyn Action> {
    match definition {
        // Movement actions
        ActionDefinition::MoveLeft { inline } => Box::new(movement::MoveLeft::new(*inline)),
        ActionDefinition::MoveRight { inline } => Box::new(movement::MoveRight::new(*inline)),
        ActionDefinition::MoveUp => Box::new(movement::MoveUp),
        ActionDefinition::MoveDown => Box::new(movement::MoveDown),
        ActionDefinition::MoveToLineStart => Box::new(movement::MoveToLineStart),
        ActionDefinition::MoveToLineEnd => Box::new(movement::MoveToLineEnd),
        ActionDefinition::MoveToTop => Box::new(movement::MoveToTop),
        ActionDefinition::MoveToBottom => Box::new(movement::MoveToBottom),
        ActionDefinition::MoveToViewportCenter => Box::new(movement::MoveToViewportCenter),
        ActionDefinition::MoveToPreviousWord => Box::new(movement::MoveToPreviousWord),
        ActionDefinition::MoveToNextWord => Box::new(movement::MoveToNextWord),
        ActionDefinition::GoToLine { line_number } => {
            Box::new(movement::GoToLine::new(*line_number))
        }

        // Editing actions
        ActionDefinition::InsertChar { ch } => Box::new(editing::InsertChar::new(*ch)),
        ActionDefinition::DeleteChar { inline } => Box::new(editing::DeleteChar::new(*inline)),
        ActionDefinition::Backspace { inline } => Box::new(editing::Backspace::new(*inline)),
        ActionDefinition::InsertNewLine => Box::new(editing::InsertNewLine),
        ActionDefinition::InsertNewLineBelow => Box::new(editing::InsertNewLineBelow),
        ActionDefinition::InsertNewLineAbove => Box::new(editing::InsertNewLineAbove),
        ActionDefinition::DeleteCurrentLine => Box::new(editing::DeleteCurrentLine),
        ActionDefinition::ChangeCurrentLine => Box::new(editing::ChangeCurrentLine),

        ActionDefinition::Undo => Box::new(editing::Undo),
        ActionDefinition::Redo => Box::new(editing::Redo),

        // Search actions
        ActionDefinition::FindNext => Box::new(search::FindNext),
        ActionDefinition::FindPrevious => Box::new(search::FindPrevious),

        // Mode actions
        ActionDefinition::EnterMode { mode } => Box::new(mode::EnterMode::new(*mode)),

        // Buffer actions
        ActionDefinition::NextBuffer => Box::new(buffer::NextBuffer),
        ActionDefinition::PreviousBuffer => Box::new(buffer::PreviousBuffer),
        ActionDefinition::OpenBuffer { path } => {
            let path_buf = PathBuf::from(path);
            Box::new(buffer::OpenBuffer::new(path_buf))
        }
        ActionDefinition::WriteBuffer { path } => {
            let path_buf = path.as_ref().map(PathBuf::from);
            Box::new(buffer::WriteBuffer::new(path_buf))
        }
        ActionDefinition::CloseBuffer { force } => Box::new(buffer::CloseBuffer::force(*force)),

        // LSP actions
        ActionDefinition::GoToDefinition => Box::new(lsp::GoToDefinition),

        // System actions
        ActionDefinition::Quit => Box::new(system::Quit),

        ActionDefinition::Composite {
            description,
            actions,
        } => {
            let mut composite = CompositeAction::new(description);
            for action_def in actions {
                composite.add(create_action_from_definition(action_def));
            }
            Box::new(composite)
        }
    }
}

pub enum MovementType {
    Line,
    Character,
}

impl ActionDefinition {
    pub fn get_movement_type(&self) -> Option<MovementType> {
        match self {
            ActionDefinition::MoveLeft { .. }
            | ActionDefinition::MoveRight { .. }
            | ActionDefinition::MoveToLineStart
            | ActionDefinition::MoveToLineEnd
            | ActionDefinition::MoveToNextWord
            | ActionDefinition::MoveToPreviousWord => Some(MovementType::Character),
            ActionDefinition::MoveUp
            | ActionDefinition::MoveDown
            | ActionDefinition::MoveToTop
            | ActionDefinition::MoveToBottom
            | ActionDefinition::GoToLine { .. } => Some(MovementType::Line),
            _ => None,
        }
    }

    pub fn is_movement_type(&self) -> bool {
        self.get_movement_type().is_some()
    }
}
