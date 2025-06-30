use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;

use crate::core::buffer_manager::BufferManager;
use crate::core::{cursor::Cursor, viewport::Viewport};
use crate::editor::Mode;
use crate::ui::components::ComponentIds;
use crate::ui::compositor::Compositor;

pub type ActionResult = Result<()>;

mod buffer;
mod editing;
mod mode;
mod movement;

pub use buffer::*;
pub use editing::*;
pub use mode::*;
pub use movement::*;

// Context passed to actions when they execute
pub struct ActionContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
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

#[macro_export]
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

    // Composite actions
    Composite {
        description: String,
        actions: Vec<ActionDefinition>,
    },

    NextBuffer,
    PreviousBuffer,
    OpenBuffer {
        path: String,
    },

    Quit,
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

        // Buffer actions
        ActionDefinition::NextBuffer => Box::new(NextBuffer),
        ActionDefinition::PreviousBuffer => Box::new(PreviousBuffer),
        ActionDefinition::OpenBuffer { path } => {
            let path_buf = PathBuf::from(path);
            Box::new(OpenBuffer::new(path_buf))
        }
        ActionDefinition::Quit => Box::new(QuitEditor),

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
