use crate::core::mode::Mode;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult, Executable};
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct EnterMode {
    mode: Mode,
}

impl EnterMode {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }
}

#[async_trait(?Send)]
impl Executable for EnterMode {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        match &ctx.mode {
            Mode::Command => {
                ctx.command_buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, false)?;
            }
            Mode::Search => {
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, false)?;
            }
            _ => {}
        };

        match &self.mode {
            Mode::Command => {
                ctx.command_buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, true)?;
                ctx.compositor.set_focus(
                    &ctx.component_ids.command_line_id,
                )?;
            }
            Mode::Search => {
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, true)?;
                ctx.compositor.set_focus(
                    &ctx.component_ids.search_box_id,
                )?;
            }
            Mode::Normal | Mode::Insert => {
                ctx.command_buffer.clear();
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .set_focus(&ctx.component_ids.editor_view_id)?;
                ctx.cursor
                    .clamp_column(ctx.buffer_manager.current_buffer(), &Mode::Normal);
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, false)?;
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, false)?;
            }
        };

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
