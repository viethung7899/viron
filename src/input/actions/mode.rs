use crate::core::mode::Mode;
use crate::core::operation::Operator;
use crate::input::actions::{
    mode, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};
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
            Mode::OperationPending => {
                ctx.input_state.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.pending_keys_id, true)?;
            }
            _ => {}
        };

        match &self.mode {
            Mode::Command => {
                ctx.command_buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, true)?;
                ctx.compositor
                    .set_focus(&ctx.component_ids.command_line_id)?;
            }
            Mode::Search => {
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, true)?;
                ctx.compositor.set_focus(&ctx.component_ids.search_box_id)?;
            }
            Mode::Normal | Mode::Insert => {
                ctx.command_buffer.clear();
                ctx.search_buffer.buffer.clear();
                ctx.compositor
                    .set_focus(&ctx.component_ids.editor_view_id)?;
                ctx.input_state.clear();
                ctx.cursor
                    .clamp_column(ctx.buffer_manager.current_buffer(), &Mode::Normal);
                ctx.compositor
                    .mark_visible(&ctx.component_ids.command_line_id, false)?;
                ctx.compositor
                    .mark_visible(&ctx.component_ids.search_box_id, false)?;
                ctx.compositor
                    .mark_visible(&ctx.component_ids.pending_keys_id, false)?;
            }
            Mode::OperationPending => {
                ctx.compositor
                    .set_focus(&ctx.component_ids.editor_view_id)?;
                ctx.compositor
                    .mark_visible(&ctx.component_ids.pending_keys_id, true)?;
            }
            _ => {}
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
            Mode::OperationPending => "Enter operation pending mode",
        }
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::EnterMode { mode: self.mode }
    }

    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct EnterPendingOperation(pub Operator);

impl EnterPendingOperation {
    pub fn new(operator: Operator) -> Self {
        Self(operator)
    }
}

#[async_trait(?Send)]
impl Executable for EnterPendingOperation {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input_state.push_operation(self.0);
        EnterMode::new(mode::Mode::OperationPending)
            .execute(ctx)
            .await
    }
}

impl Action for EnterPendingOperation {
    fn describe(&self) -> &str {
        match self.0 {
            Operator::Delete => "Delete",
            Operator::Change => "Change",
        }
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::EnterPendingOperation { operator: self.0 }
    }

    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}
