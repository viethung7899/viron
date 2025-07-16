use crate::actions::core::definition::{MovementType, create_action_from_definition};
use crate::actions::core::{ActionDefinition, Executable};
use crate::actions::types::editing::after_edit;
use crate::actions::types::{editing, mode, movement};
use crate::actions::{ActionContext, ActionResult};
use crate::core::history::edit::Edit;
use crate::core::mode::Mode;
use crate::core::operation::Operator;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RepeatingAction {
    repeat: usize,
    action: ActionDefinition,
}

impl RepeatingAction {
    pub fn new(repeat: usize, action: ActionDefinition) -> Self {
        Self { repeat, action }
    }
}

#[async_trait(?Send)]
impl Executable for RepeatingAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let action = create_action_from_definition(&self.action);
        for _ in 0..self.repeat {
            action.execute(ctx).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ComboAction {
    operator: Operator,
    repeat: usize,
    motion: ActionDefinition,
}

impl ComboAction {
    pub fn new(operator: Operator, repeat: usize, motion: ActionDefinition) -> Self {
        Self {
            operator,
            repeat,
            motion,
        }
    }

    async fn perform_delete(&self, ctx: &mut ActionContext<'_>) -> anyhow::Result<bool> {
        let movement_type = self.motion.get_movement_type().unwrap();
        let before = ctx.cursor.get_point();
        let action = create_action_from_definition(&self.motion);
        for _ in 0..self.repeat {
            action.execute(ctx).await?;
        }
        let after = ctx.cursor.get_point();

        let from = before.min(after);
        let to = before.max(after);

        let buffer = ctx.buffer_manager.current_buffer_mut();
        let result = match movement_type {
            MovementType::Line => {
                let start_line = from.row;
                let end_line = to.row;
                buffer.delete_multiple_lines(start_line, end_line)
            }
            MovementType::Character => {
                let start = buffer.cursor_position(&from);
                let end = buffer.cursor_position(&to);
                buffer.delete_string(start, end - start)
            }
        };

        let Some((deleted, start_byte)) = result else {
            return Ok(false);
        };

        let edit = Edit::delete(
            start_byte,
            buffer.point_at_position(start_byte),
            deleted,
            from,
            to,
        );
        ctx.cursor.set_point(from, buffer);
        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
        Ok(true)
    }

    async fn perform_change(&self, ctx: &mut ActionContext<'_>) -> ActionResult {
        let movement_type = self.motion.get_movement_type().unwrap();
        let deleted = self.perform_delete(ctx).await?;
        match movement_type {
            MovementType::Line if deleted => {
                editing::InsertNewLineAbove.execute(ctx).await?;
            }
            _ => {
                mode::EnterMode::new(Mode::Insert).execute(ctx).await?;
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Executable for ComboAction {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !self.motion.is_movement_type() {
            return Ok(());
        };

        match self.operator {
            Operator::Delete => {
                self.perform_delete(ctx).await?;
            }
            Operator::Change => self.perform_change(ctx).await?,
        };
        let buffer = ctx.buffer_manager.current_buffer();
        ctx.cursor.clamp_row(buffer);
        ctx.cursor.clamp_column(buffer, ctx.mode);
        Ok(())
    }
}
