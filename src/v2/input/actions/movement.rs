use std::fmt::Debug;
use crate::impl_action;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionImpl, ActionResult};

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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move cursor left"
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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move cursor right"
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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move cursor up"
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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move cursor down"
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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move to start of line"
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
        Ok(())
    }

    fn describe_impl(&self) -> &str {
        "Move to end of line"
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineEnd
    }
}

impl_action!(MoveLeft);
impl_action!(MoveRight);
impl_action!(MoveUp);
impl_action!(MoveDown);
impl_action!(MoveToLineStart);
impl_action!(MoveToLineEnd);

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
