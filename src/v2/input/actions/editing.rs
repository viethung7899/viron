use crate::editor::Mode;
use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct InsertChar(char);

impl InsertChar {
    pub fn new(ch: char) -> Self {
        Self(ch)
    }
}

impl Executable for InsertChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, self.0);
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        ctx.buffer_manager.current_mut().mark_modified();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}
impl_action!(InsertChar, "Insert char", self {
    ActionDefinition::InsertChar { ch: self.0 }
});

#[derive(Debug, Clone)]
pub struct DeleteChar;

impl Executable for DeleteChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        if let Some(_) = buffer.delete_char(position) {
            // Cursor stays in place after deletion
            ctx.buffer_manager.current_mut().mark_modified();
            ctx.compositor
                .mark_dirty(&ctx.component_ids.buffer_view_id)?;
            ctx.compositor
                .mark_dirty(&ctx.component_ids.status_line_id)?;
        }
        Ok(())
    }
}

impl_action!(DeleteChar, "Delete character", self {
    ActionDefinition::DeleteChar
});

#[derive(Debug, Clone)]
pub struct Backspace;

impl Executable for Backspace {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        ctx.cursor.move_left(buffer, ctx.mode);
        if position > 0 {
            if let Some(_) = buffer.delete_char(position - 1) {
                ctx.buffer_manager.current_mut().mark_modified();
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.buffer_view_id)?;
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.status_line_id)?;
            }
        }
        Ok(())
    }
}

impl_action!(Backspace, "Backspace", self {
    ActionDefinition::Backspace
});

#[derive(Debug, Clone)]
pub struct InsertNewLine;

impl Executable for InsertNewLine {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&ctx.cursor.get_position());
        let new_position = buffer.insert_char(position, '\n');
        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        ctx.buffer_manager.current_mut().mark_modified();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(InsertNewLine, "Insert new line", self {
    ActionDefinition::InsertNewLine
});

#[derive(Debug, Clone)]
pub struct InsertNewLineBelow;

impl Executable for InsertNewLineBelow {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let cursor = ctx.cursor.get_position();

        // Get the indentation of the current line
        let current_line = buffer.get_content_line(cursor.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to end of the current line
        ctx.cursor.move_to_line_end(buffer, &Mode::Insert);
        let position = buffer.cursor_position(&ctx.cursor.get_position());

        // Insert newline at end of current line followed by indentation
        let insert_text = format!("\n{}", indentation);
        let new_position = buffer.insert_string(position, &insert_text);

        ctx.cursor
            .set_position(buffer.point_at_position(new_position));
        ctx.buffer_manager.current_mut().mark_modified();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(InsertNewLineBelow, "Insert new line below", self {
    ActionDefinition::InsertNewLineBelow
});

#[derive(Debug, Clone)]
pub struct InsertNewLineAbove;

impl Executable for InsertNewLineAbove {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let cursor_pos = ctx.cursor.get_position();

        // Get the indentation of the current line
        let current_line = buffer.get_content_line(cursor_pos.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to beginning of current line
        ctx.cursor.move_to_line_start();
        let position = buffer.cursor_position(&ctx.cursor.get_position());

        // Insert indentation followed by newline
        let insert_text = format!("{}\n", indentation);
        buffer.insert_string(position, &insert_text);

        // Position cursor at end of the new line (after indentation)
        let cursor_position = position + indentation.len();
        ctx.cursor
            .set_position(buffer.point_at_position(cursor_position));
        ctx.buffer_manager.current_mut().mark_modified();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(InsertNewLineAbove, "Insert new line above", self {
    ActionDefinition::InsertNewLineAbove
});
