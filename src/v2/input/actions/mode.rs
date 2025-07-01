use std::fmt::Debug;

use crate::editor::Mode;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult, Executable};

#[derive(Debug, Clone)]
pub struct EnterMode {
    mode: Mode,
}

impl EnterMode {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }
}

impl Executable for EnterMode {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        match (&ctx.mode, &self.mode) {
            (Mode::Command, _) => {
                ctx.command_buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, false)?;
            }
            (Mode::Search, _) => {
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, false)?;
            }
            (_, Mode::Command) => {
                ctx.command_buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, true)?;
            }
            (_, Mode::Search) => {
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, true)?;
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.search_box_id)?;
            }
            (_, Mode::Normal) => {
                ctx.command_buffer.clear();
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, false)?;
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, false)?;
            }
            _ => {}
        }
        *ctx.mode = self.mode.clone();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl Action for EnterMode {
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
