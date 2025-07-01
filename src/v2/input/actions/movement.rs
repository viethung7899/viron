use crate::input::actions::{impl_action, Action, ActionContext, ActionDefinition, ActionImpl, ActionResult};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct MoveLeft {
    count: usize,
}

impl MoveLeft {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl ActionImpl for MoveLeft {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_left(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveLeft { count: self.count }
    }
}

#[derive(Debug, Clone)]
pub struct MoveRight {
    count: usize,
}

impl MoveRight {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl ActionImpl for MoveRight {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_right(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveRight { count: self.count }
    }
}

#[derive(Debug, Clone)]
pub struct MoveUp {
    count: usize,
}

impl MoveUp {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl ActionImpl for MoveUp {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_up(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveUp { count: self.count }
    }
}

#[derive(Debug, Clone)]
pub struct MoveDown {
    count: usize,
}

impl MoveDown {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl ActionImpl for MoveDown {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_down(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveDown { count: self.count }
    }
}

#[derive(Debug, Clone)]
pub struct MoveToLineStart;

impl ActionImpl for MoveToLineStart {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_line_start();
        ctx.cursor.find_next_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineStart
    }
}

#[derive(Debug, Clone)]
pub struct MoveToLineEnd;

impl ActionImpl for MoveToLineEnd {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .move_to_line_end(ctx.buffer_manager.current_buffer(), ctx.mode);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineEnd
    }
}

#[derive(Debug, Clone)]
pub struct MoveToTop;

impl ActionImpl for MoveToTop {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_top();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToTop
    }
}

#[derive(Debug, Clone)]
pub struct MoveToBottom;

impl ActionImpl for MoveToBottom {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .move_to_bottom(ctx.buffer_manager.current_buffer(), ctx.mode);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToBottom
    }
}

#[derive(Debug, Clone)]
pub struct MoveToViewportCenter;

impl ActionImpl for MoveToViewportCenter {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.viewport.center_on_line(
            ctx.cursor.get_position().row,
            ctx.buffer_manager.current_buffer(),
        );
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor.mark_dirty(&ctx.component_ids.gutter_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToViewportCenter
    }
}

#[derive(Debug, Clone)]
pub struct MoveToNextWord;

impl ActionImpl for MoveToNextWord {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .find_next_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToNextWord
    }
}

#[derive(Debug, Clone)]
pub struct MoveToPreviousWord;

impl ActionImpl for MoveToPreviousWord {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .find_previous_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToPreviousWord
    }
}

impl_action!(MoveLeft, "Move cursor left");
impl_action!(MoveRight, "Move cursor right");
impl_action!(MoveUp, "Move cursor up");
impl_action!(MoveDown, "Move cursor down");
impl_action!(MoveToLineStart, "Move to line start");
impl_action!(MoveToLineEnd, "Move to line end");
impl_action!(MoveToTop, "Move to top of buffer");
impl_action!(MoveToBottom, "Move to bottom of buffer");
impl_action!(MoveToViewportCenter, "Move viewport to center of buffer");
impl_action!(MoveToNextWord, "Move cursor to next word");
impl_action!(MoveToPreviousWord, "Move cursor to previous word");
