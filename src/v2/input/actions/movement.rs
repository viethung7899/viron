use crate::impl_action;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionImpl, ActionResult};
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
pub struct MoveToLineStart {}

impl ActionImpl for MoveToLineStart {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_line_start();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineStart
    }
}

#[derive(Debug, Clone)]
pub struct MoveToLineEnd {}

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

impl_action!(MoveLeft, "Move cursor left");
impl_action!(MoveRight, "Move cursor right");
impl_action!(MoveUp, "Move cursor up");
impl_action!(MoveDown, "Move cursor down");
impl_action!(MoveToLineStart, "Move to line start");
impl_action!(MoveToLineEnd, "Move to line end");

// Convenience functions for creating movement actions
pub fn move_left(count: usize) -> Box<dyn Action> {
    Box::new(MoveLeft::new(count))
}

pub fn move_right(count: usize) -> Box<dyn Action> {
    Box::new(MoveRight::new(count))
}

pub fn move_up(count: usize) -> Box<dyn Action> {
    Box::new(MoveUp::new(count))
}

pub fn move_down(count: usize) -> Box<dyn Action> {
    Box::new(MoveDown::new(count))
}

pub fn move_to_line_start() -> Box<dyn Action> {
    Box::new(MoveToLineStart {})
}

pub fn move_to_line_end() -> Box<dyn Action> {
    Box::new(MoveToLineEnd {})
}
