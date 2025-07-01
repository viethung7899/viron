use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;

use crate::core::buffer_manager::BufferManager;
use crate::core::command_buffer::CommandBuffer;
use crate::core::{cursor::Cursor, viewport::Viewport};
use crate::editor::Mode;
use crate::ui::components::ComponentIds;
use crate::ui::compositor::Compositor;

pub type ActionResult = Result<()>;

mod buffer;
mod command;
mod editing;
mod mode;
mod movement;
mod system;

pub use buffer::*;
pub use command::*;
pub use editing::*;
pub use mode::*;
pub use movement::*;
pub use system::*;

// Context passed to actions when they execute
pub struct ActionContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub command_buffer: &'a mut CommandBuffer,
    pub cursor: &'a mut Cursor,
    pub viewport: &'a mut Viewport,
    pub mode: &'a mut Mode,
    pub running: &'a mut bool,

    pub compositor: &'a mut Compositor,
    pub component_ids: &'a ComponentIds,
}

// The Action trait defines what all actions must implement
pub trait Action: Debug + Send + Sync {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult;
    fn describe(&self) -> &str;
    fn to_serializable(&self) -> ActionDefinition;
    fn clone_box(&self) -> Box<dyn Action>;
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// A composite action that runs multiple actions in sequence
#[derive(Debug, Clone)]
pub struct CompositeAction {
    actions: Vec<Box<dyn Action>>,
    description: String,
}

impl CompositeAction {
    pub fn new(description: &str) -> Self {
        Self {
            actions: Vec::new(),
            description: description.to_string(),
        }
    }

    pub fn add(&mut self, action: Box<dyn Action>) -> &mut Self {
        self.actions.push(action);
        self
    }
}

impl Action for CompositeAction {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for action in &self.actions {
            action.execute(ctx)?;
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        &self.description
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::Composite {
            description: self.description.clone(),
            actions: self
                .actions
                .iter()
                .map(|action| action.to_serializable())
                .collect(),
        }
    }
    fn clone_box(&self) -> Box<(dyn Action + 'static)> {
        Box::new(self.clone())
    }
}

pub(super) trait ActionImpl: Debug + Clone + Send + Sync {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult;
    fn to_serializable_impl(&self) -> ActionDefinition;
}

macro_rules! impl_action {
    ($action_type:ty, $description:expr) => {
        impl Action for $action_type {
            fn clone_box(&self) -> Box<dyn Action> {
                Box::new(self.clone())
            }

            // Other methods must still be implemented manually
            fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
                self.execute_impl(ctx)
            }

            fn describe(&self) -> &str {
                $description
            }

            fn to_serializable(&self) -> ActionDefinition {
                self.to_serializable_impl()
            }
        }
    };
}

pub(super) use impl_action;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum ActionDefinition {
    // Movement actions
    MoveLeft {
        count: usize,
    },
    MoveRight {
        count: usize,
    },
    MoveUp {
        count: usize,
    },
    MoveDown {
        count: usize,
    },
    MoveToLineStart,
    MoveToLineEnd,
    MoveToTop,
    MoveToBottom,
    MoveToViewportCenter,
    MoveToPreviousWord,
    MoveToNextWord,

    // Editing actions
    InsertChar {
        ch: char,
    },
    DeleteChar,
    Backspace,
    InsertNewLine,
    InsertNewLineBelow,
    InsertNewLineAbove,

    // Mode actions
    EnterMode {
        mode: String,
    },

    // Command actions
    CommandMoveLeft,
    CommandMoveRight,
    CommandInsertChar {
        ch: char,
    },
    CommandDeleteChar,
    CommandBackspace,
    CommandExecute,

    // Buffer actions
    NextBuffer,
    PreviousBuffer,
    OpenBuffer {
        path: String,
    },
    WriteBuffer {
        path: Option<String>,
    },

    // System actions
    Quit {
        force: bool,
    },

    // Composite actions
    Composite {
        description: String,
        actions: Vec<ActionDefinition>,
    },
}

pub fn create_action_from_definition(definition: &ActionDefinition) -> Box<dyn Action> {
    match definition {
        // Movement actions
        ActionDefinition::MoveLeft { count } => Box::new(MoveLeft::new(*count)),
        ActionDefinition::MoveRight { count } => Box::new(MoveRight::new(*count)),
        ActionDefinition::MoveUp { count } => Box::new(MoveUp::new(*count)),
        ActionDefinition::MoveDown { count } => Box::new(MoveDown::new(*count)),
        ActionDefinition::MoveToLineStart => Box::new(MoveToLineStart),
        ActionDefinition::MoveToLineEnd => Box::new(MoveToLineEnd),
        ActionDefinition::MoveToTop => Box::new(MoveToTop),
        ActionDefinition::MoveToBottom => Box::new(MoveToBottom),
        ActionDefinition::MoveToViewportCenter => Box::new(MoveToViewportCenter),
        ActionDefinition::MoveToPreviousWord => Box::new(MoveToPreviousWord),
        ActionDefinition::MoveToNextWord => Box::new(MoveToNextWord),

        // Editing actions
        ActionDefinition::InsertChar { ch } => Box::new(InsertChar::new(*ch)),
        ActionDefinition::DeleteChar => Box::new(DeleteChar),
        ActionDefinition::Backspace => Box::new(Backspace),
        ActionDefinition::InsertNewLine => Box::new(InsertNewLine),
        ActionDefinition::InsertNewLineBelow => Box::new(InsertNewLineBelow),
        ActionDefinition::InsertNewLineAbove => Box::new(InsertNewLineAbove),

        // Mode actions
        ActionDefinition::EnterMode { mode } => {
            let mode = match mode.as_str() {
                "insert" => Mode::Insert,
                "command" => Mode::Command,
                "search" => Mode::Search,
                _ => Mode::Normal, // Default fallback
            };
            Box::new(EnterMode::new(mode))
        }

        // Command actions
        ActionDefinition::CommandMoveLeft => Box::new(CommandMoveLeft),
        ActionDefinition::CommandMoveRight => Box::new(CommandMoveRight),
        ActionDefinition::CommandInsertChar { ch } => Box::new(CommandInsertChar::new(*ch)),
        ActionDefinition::CommandDeleteChar => Box::new(CommandDeleteChar),
        ActionDefinition::CommandBackspace => Box::new(CommandBackspace),
        ActionDefinition::CommandExecute => Box::new(CommandExecute),

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
        
        // System actions
        ActionDefinition::Quit { force } => Box::new(QuitEditor::new(*force)),

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
