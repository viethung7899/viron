use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::core::{buffer::Buffer, cursor::Cursor, document::Document, viewport::Viewport};
use crate::editor::Mode;

pub type ActionResult = Result<()>;

mod editing;
mod mode;
mod movement;

pub use editing::*;
pub use mode::*;
pub use movement::*;

// Context passed to actions when they execute
pub struct ActionContext<'a> {
    pub document: &'a mut Document,
    pub buffer: &'a mut Buffer,
    pub cursor: &'a mut Cursor,
    pub viewport: &'a mut Viewport,
    pub mode: &'a mut Mode,
}

// The Action trait defines what all actions must implement
pub trait Action: Debug + Send + Sync {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult;
    fn describe(&self) -> &str;

    fn to_serializable(&self) -> ActionDefinition;
}

// A composite action that runs multiple actions in sequence
#[derive(Debug)]
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

    // Editing actions
    InsertChar {
        ch: char,
    },
    DeleteChar,
    Backspace,
    InsertNewLine,

    // Mode actions
    EnterMode {
        mode: String,
    },

    // Composite actions
    Composite {
        description: String,
        actions: Vec<ActionDefinition>,
    },
}

pub fn create_action_from_definition(definition: &ActionDefinition) -> Box<dyn Action> {
    match definition {
        ActionDefinition::MoveLeft { count } => move_left(*count),
        ActionDefinition::MoveRight { count } => move_right(*count),
        ActionDefinition::MoveUp { count } => move_up(*count),
        ActionDefinition::MoveDown { count } => move_down(*count),
        ActionDefinition::MoveToLineStart {} => move_to_line_start(),
        ActionDefinition::MoveToLineEnd {} => move_to_line_end(),

        ActionDefinition::InsertChar { ch } => insert_char(*ch),
        ActionDefinition::DeleteChar {} => delete_char(),
        ActionDefinition::Backspace {} => backspace(),
        ActionDefinition::InsertNewLine {} => insert_new_line(),

        ActionDefinition::EnterMode { mode } => match mode.as_str() {
            "normal" => enter_normal_mode(),
            "insert" => enter_insert_mode(),
            "command" => enter_command_mode(),
            "search" => enter_search_mode(),
            _ => enter_normal_mode(), // Default fallback
        },

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
