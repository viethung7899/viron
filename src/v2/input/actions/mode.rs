use std::fmt::Debug;

use crate::editor::Mode;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult};

#[derive(Debug)]
pub struct EnterMode {
    mode: Mode,
}

impl EnterMode {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }
}

impl Action for EnterMode {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        *ctx.mode = self.mode.clone();
        Ok(())
    }

    fn describe(&self) -> &str {
        match self.mode {
            Mode::Normal => "Enter normal mode",
            Mode::Insert => "Enter insert mode",
            Mode::Command => "Enter command mode",
            Mode::Search => "Enter search mode",
        }
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::EnterMode {
            mode: self.mode.to_string(),
        }
    }
}

// Convenience functions for mode switching
pub fn enter_normal_mode() -> Box<dyn Action> {
    Box::new(EnterMode::new(Mode::Normal))
}

pub fn enter_insert_mode() -> Box<dyn Action> {
    Box::new(EnterMode::new(Mode::Insert))
}

pub fn enter_command_mode() -> Box<dyn Action> {
    Box::new(EnterMode::new(Mode::Command))
}

pub fn enter_search_mode() -> Box<dyn Action> {
    Box::new(EnterMode::new(Mode::Search))
}
