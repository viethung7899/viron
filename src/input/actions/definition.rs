use crate::core::mode::Mode;
use crate::input::actions::composite::CompositeAction;
use crate::input::actions::{
    Action, Backspace, CloseBuffer, DeleteChar, DeleteCurrentLine, DeleteWord, EnterMode, FindNext,
    FindPrevious, GoToDefinition, GoToLine, InsertChar, InsertNewLine, InsertNewLineAbove,
    InsertNewLineBelow, MoveDown, MoveLeft, MoveRight, MoveToBottom, MoveToLineEnd,
    MoveToLineStart, MoveToNextWord, MoveToPreviousWord, MoveToTop, MoveToViewportCenter, MoveUp,
    NextBuffer, OpenBuffer, PreviousBuffer, Quit, Redo, Undo, WriteBuffer,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum ActionDefinition {
    // Movement actions
    MoveLeft {
        previous_line: bool,
    },
    MoveRight {
        next_line: bool,
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

    Backspace,
    DeleteChar,
    DeleteCurrentLine,
    DeleteWord,

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
        ActionDefinition::MoveLeft { previous_line } => Box::new(MoveLeft::new(*previous_line)),
        ActionDefinition::MoveRight { next_line } => Box::new(MoveRight::new(*next_line)),
        ActionDefinition::MoveUp => Box::new(MoveUp),
        ActionDefinition::MoveDown => Box::new(MoveDown),
        ActionDefinition::MoveToLineStart => Box::new(MoveToLineStart),
        ActionDefinition::MoveToLineEnd => Box::new(MoveToLineEnd),
        ActionDefinition::MoveToTop => Box::new(MoveToTop),
        ActionDefinition::MoveToBottom => Box::new(MoveToBottom),
        ActionDefinition::MoveToViewportCenter => Box::new(MoveToViewportCenter),
        ActionDefinition::MoveToPreviousWord => Box::new(MoveToPreviousWord),
        ActionDefinition::MoveToNextWord => Box::new(MoveToNextWord),
        ActionDefinition::GoToLine { line_number } => Box::new(GoToLine::new(*line_number)),

        // Editing actions
        ActionDefinition::InsertChar { ch } => Box::new(InsertChar::new(*ch)),
        ActionDefinition::DeleteChar => Box::new(DeleteChar),
        ActionDefinition::Backspace => Box::new(Backspace),
        ActionDefinition::InsertNewLine => Box::new(InsertNewLine),
        ActionDefinition::InsertNewLineBelow => Box::new(InsertNewLineBelow),
        ActionDefinition::InsertNewLineAbove => Box::new(InsertNewLineAbove),
        ActionDefinition::DeleteCurrentLine => Box::new(DeleteCurrentLine),
        ActionDefinition::DeleteWord => Box::new(DeleteWord),

        ActionDefinition::Undo => Box::new(Undo),
        ActionDefinition::Redo => Box::new(Redo),

        // Search actions
        ActionDefinition::FindNext => Box::new(FindNext),
        ActionDefinition::FindPrevious => Box::new(FindPrevious),

        // Mode actions
        ActionDefinition::EnterMode { mode } => Box::new(EnterMode::new(*mode)),

        // Buffer actions
        ActionDefinition::NextBuffer => Box::new(NextBuffer),
        ActionDefinition::PreviousBuffer => Box::new(PreviousBuffer),
        ActionDefinition::OpenBuffer { path } => {
            let path_buf = PathBuf::from(path);
            Box::new(OpenBuffer::new(path_buf))
        }
        ActionDefinition::WriteBuffer { path } => {
            let path_buf = path.as_ref().map(PathBuf::from);
            Box::new(WriteBuffer::new(path_buf))
        }
        ActionDefinition::CloseBuffer { force } => Box::new(CloseBuffer::force(*force)),

        // LSP actions
        ActionDefinition::GoToDefinition => Box::new(GoToDefinition),

        // System actions
        ActionDefinition::Quit => Box::new(Quit),

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
