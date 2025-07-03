use crate::core::history::change::Change;
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
        let current_point = ctx.cursor.get_point();

        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&current_point);

        let new_position = buffer.insert_char(position, self.0);
        let new_point = buffer.point_at_position(new_position);

        ctx.cursor.set_point(new_point.clone());

        let document = ctx.buffer_manager.current_mut();
        document.mark_modified();

        document.history.push(Change::insert(
            position,
            self.0.to_string(),
            current_point,
            new_point,
        ));

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
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();
        let position = document.buffer.cursor_position(&point);
        if let Some((c, new_point)) = document.buffer.delete_char(position) {
            // Cursor stays in place after deletion
            document.mark_modified();
            document
                .history
                .push(Change::delete(position, c.to_string(), point, point));
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
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();
        let position = document.buffer.cursor_position(&point);
        ctx.cursor.move_left(&document.buffer, ctx.mode);
        if position > 0 {
            if let Some((c, new_position)) = document.buffer.delete_char(position - 1) {
                document.mark_modified();
                document.history.push(Change::delete(
                    position - 1,
                    c.to_string(),
                    point,
                    document.buffer.point_at_position(new_position),
                ));
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
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();
        let position = document.buffer.cursor_position(&point);
        let new_position = document.buffer.insert_char(position, '\n');
        let new_point = document.buffer.point_at_position(new_position);
        document.mark_modified();
        document.history.push(Change::insert(
            position,
            "\n".to_string(),
            point,
            new_point.clone(),
        ));
        ctx.cursor.set_point(new_point);
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
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();

        // Get the indentation of the current line
        let current_line = document.buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to end of the current line
        ctx.cursor.move_to_line_end(&document.buffer, &Mode::Insert);
        let position = document.buffer.cursor_position(&ctx.cursor.get_point());

        // Insert newline at end of current line followed by indentation
        let insert_text = format!("\n{}", indentation);
        let new_position = document.buffer.insert_string(position, &insert_text);
        let new_point = document.buffer.point_at_position(new_position);

        ctx.cursor.set_point(new_point);
        document
            .history
            .push(Change::insert(position, insert_text, point, new_point));
        document.mark_modified();
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
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();

        // Get the indentation of the current line
        let current_line = document.buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to beginning of current line
        ctx.cursor.move_to_line_start();
        let position = document.buffer.cursor_position(&ctx.cursor.get_point());

        // Insert indentation followed by newline
        let insert_text = format!("{}\n", indentation);
        document.buffer.insert_string(position, &insert_text);

        // Position cursor at end of the new line (after indentation)
        let cursor_position = position + indentation.len();
        let new_point = document.buffer.point_at_position(cursor_position);

        ctx.cursor.set_point(new_point);

        document.mark_modified();
        document
            .history
            .push(Change::insert(position, insert_text, point, new_point));
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
