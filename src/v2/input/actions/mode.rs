use std::fmt::Debug;

use crate::editor::Mode;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult};

#[derive(Debug, Clone)]
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
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
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

    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}
