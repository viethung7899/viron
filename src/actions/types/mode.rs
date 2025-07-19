use crate::actions::core::{Action, ActionDefinition, Executable};
use crate::actions::ActionResult;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use async_trait::async_trait;
use std::fmt::Debug;
use crate::actions::context::ActionContext;

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
        match &ctx.editor.mode {
            Mode::Command => {
                ctx.input.command_buffer.clear();
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.command_line_id, false)?;
            }
            Mode::Search => {
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.search_box_id, false)?;
            }
            Mode::OperationPending(_) => {
                ctx.input.input_state.clear();
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.pending_keys_id, false)?;
            }
            _ => {}
        };

        match &self.mode {
            Mode::Command => {
                ctx.input.command_buffer.clear();
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.command_line_id, true)?;
                ctx.ui.compositor
                    .set_focus(&ctx.ui.component_ids.command_line_id)?;
            }
            Mode::Search => {
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.search_box_id, true)?;
                ctx.ui.compositor.set_focus(&ctx.ui.component_ids.search_box_id)?;
            }
            Mode::Normal | Mode::Insert => {
                ctx.input.command_buffer.clear();
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .set_focus(&ctx.ui.component_ids.editor_view_id)?;
                ctx.input.input_state.clear();
                ctx.editor.cursor
                    .clamp_column(ctx.editor.buffer_manager.current_buffer(), &Mode::Normal);
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.command_line_id, false)?;
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.search_box_id, false)?;
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.pending_keys_id, false)?;
            }
            Mode::OperationPending(_) => {
                ctx.ui.compositor
                    .set_focus(&ctx.ui.component_ids.editor_view_id)?;
                ctx.ui.compositor
                    .mark_visible(&ctx.ui.component_ids.pending_keys_id, true)?;
            }
        };

        *ctx.editor.mode = self.mode.clone();
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
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
            Mode::OperationPending(Operator::Change) => "Change",
            Mode::OperationPending(Operator::Delete) => "Delete",
        }
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::EnterMode { mode: self.mode }
    }

    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}
