use std::fmt::Debug;

use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult};

#[derive(Debug)]
pub struct InsertChar(char);

impl Action for InsertChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, self.0);
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        Ok(())
    }

    fn describe(&self) -> &str {
        "Insert character"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::InsertChar { ch: self.0 }
    }
}

#[derive(Debug)]
pub struct DeleteChar;

impl Action for DeleteChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        if let Some(_) = buffer.delete_char(position) {
            // Cursor stays in place after deletion
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Delete character"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::DeleteChar
    }
}

#[derive(Debug)]
pub struct Backspace;

impl Action for Backspace {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        if position > 0 {
            if let Some(_) = buffer.delete_char(position - 1) {
                ctx.cursor.move_left(buffer, ctx.mode);
            }
        }
        Ok(())
    }

    fn describe(&self) -> &str {
        "Delete previous character"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::Backspace
    }
}

#[derive(Debug)]
pub struct InsertNewLine;

impl Action for InsertNewLine {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, '\n');
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        Ok(())
    }

    fn describe(&self) -> &str {
        "Insert new line"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::InsertNewLine
    }
}

// Convenience functions for creating editing actions
pub fn insert_char(ch: char) -> Box<dyn Action> {
    Box::new(InsertChar(ch))
}

pub fn delete_char() -> Box<dyn Action> {
    Box::new(DeleteChar)
}

pub fn backspace() -> Box<dyn Action> {
    Box::new(Backspace)
}

pub fn insert_new_line() -> Box<dyn Action> {
    Box::new(InsertNewLine)
}
