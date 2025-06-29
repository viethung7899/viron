use crate::impl_action;
use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionImpl, ActionResult};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct InsertChar(char);

impl ActionImpl for InsertChar {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, self.0);
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::InsertChar { ch: self.0 }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteChar;

impl ActionImpl for DeleteChar {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        if let Some(_) = buffer.delete_char(position) {
            // Cursor stays in place after deletion
        }
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::DeleteChar
    }
}

#[derive(Debug, Clone)]
pub struct Backspace;

impl ActionImpl for Backspace {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        if position > 0 {
            if let Some(_) = buffer.delete_char(position - 1) {
                ctx.cursor.move_left(buffer, ctx.mode);
            }
        }
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::Backspace
    }
}

#[derive(Debug, Clone)]
pub struct InsertNewLine;

impl ActionImpl for InsertNewLine {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, '\n');
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::InsertNewLine
    }
}

impl_action!(InsertNewLine, "Insert new line");
impl_action!(Backspace, "Backspace");
impl_action!(DeleteChar, "Delete character");
impl_action!(InsertChar, "Insert new line");

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
