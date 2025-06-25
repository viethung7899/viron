use anyhow::Result;
use std::fmt::Debug;

use crate::core::{buffer::Buffer, cursor::Cursor};
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult};

#[derive(Debug)]
pub struct MoveLeft {
    count: usize,
}

impl MoveLeft {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Action for MoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_left(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move cursor left"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveLeft { count: self.count }
    }
}

#[derive(Debug)]
pub struct MoveRight {
    count: usize,
}

impl MoveRight {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Action for MoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_right(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move cursor right"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveRight { count: self.count }
    }
}

#[derive(Debug)]
pub struct MoveUp {
    count: usize,
}

impl MoveUp {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Action for MoveUp {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_up(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move cursor up"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveUp { count: self.count }
    }
}

#[derive(Debug)]
pub struct MoveDown {
    count: usize,
}

impl MoveDown {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Action for MoveDown {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_down(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move cursor down"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveDown { count: self.count }
    }
}

#[derive(Debug)]
pub struct MoveToLineStart {}

impl Action for MoveToLineStart {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_line_start();
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move to start of line"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineStart
    }
}

#[derive(Debug)]
pub struct MoveToLineEnd {}

impl Action for MoveToLineEnd {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .move_to_line_end(ctx.buffer_manager.current_buffer(), ctx.mode);
        Ok(())
    }

    fn describe(&self) -> &str {
        "Move to end of line"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::MoveToLineEnd
    }
}

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
