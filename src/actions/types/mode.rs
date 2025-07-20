use crate::actions::core::{Action, ActionDefinition, Executable};
use crate::actions::ActionResult;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use async_trait::async_trait;
use std::fmt::Debug;
use crate::actions::context::ActionContext;
use crate::constants::components::{COMMAND_LINE, EDITOR_VIEW, PENDING_KEYS, SEARCH_BOX, STATUS_LINE};

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
                    .mark_visible(COMMAND_LINE, false)?;
            }
            Mode::Search => {
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .mark_visible(SEARCH_BOX, false)?;
            }
            Mode::OperationPending(_) => {
                ctx.input.input_state.clear();
                ctx.ui.compositor
                    .mark_visible(PENDING_KEYS, false)?;
            }
            _ => {}
        };

        match &self.mode {
            Mode::Command => {
                ctx.input.command_buffer.clear();
                ctx.ui.compositor
                    .mark_visible(COMMAND_LINE, true)?;
                ctx.ui.compositor
                    .set_focus(COMMAND_LINE)?;
            }
            Mode::Search => {
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .mark_visible(SEARCH_BOX, true)?;
                ctx.ui.compositor.set_focus(SEARCH_BOX)?;
            }
            Mode::Normal | Mode::Insert => {
                ctx.input.command_buffer.clear();
                ctx.input.search_buffer.buffer.clear();
                ctx.ui.compositor
                    .set_focus(EDITOR_VIEW)?;
                ctx.input.input_state.clear();
                ctx.editor.cursor
                    .clamp_column(ctx.editor.buffer_manager.current_buffer(), &Mode::Normal);
                ctx.ui.compositor
                    .mark_visible(COMMAND_LINE, false)?;
                ctx.ui.compositor
                    .mark_visible(SEARCH_BOX, false)?;
                ctx.ui.compositor
                    .mark_visible(PENDING_KEYS, false)?;
            }
            Mode::OperationPending(_) => {
                ctx.ui.compositor
                    .set_focus(EDITOR_VIEW)?;
                ctx.ui.compositor
                    .mark_visible(PENDING_KEYS, true)?;
            }
        };

        *ctx.editor.mode = self.mode.clone();
        ctx.ui.compositor
            .mark_dirty(STATUS_LINE)?;
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
            Mode::OperationPending(Operator::Yank) => "Yank",
        }
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::EnterMode { mode: self.mode }
    }

    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}
