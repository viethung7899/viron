use crate::core::history::change::Change;
use crate::editor::Mode;
use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct InsertChar(char);

impl InsertChar {
    pub fn new(ch: char) -> Self {
        Self(ch)
    }
}

#[async_trait(?Send)]
impl Executable for InsertChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let current_point = ctx.cursor.get_point();

        let buffer = ctx.buffer_manager.current_buffer_mut();
        let position = buffer.cursor_position(&current_point);

        let new_position = buffer.insert_char(position, self.0);
        let new_point = buffer.point_at_position(new_position);

        ctx.cursor.set_point(new_point.clone(), buffer);

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
        ctx.search_buffer.reset();
        Ok(())
    }
}
impl_action!(InsertChar, "Insert char", self {
    ActionDefinition::InsertChar { ch: self.0 }
});

#[derive(Debug, Clone)]
pub struct DeleteChar;

#[async_trait(?Send)]
impl Executable for DeleteChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();
        let position = document.buffer.cursor_position(&point);
        if let Some((c, _)) = document.buffer.delete_char(position) {
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
        ctx.search_buffer.reset();
        Ok(())
    }
}

impl_action!(DeleteChar, "Delete character", ActionDefinition::DeleteChar);

#[derive(Debug, Clone)]
pub struct Backspace;

#[async_trait(?Send)]
impl Executable for Backspace {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
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
                ctx.search_buffer.reset();
            }
        }
        Ok(())
    }
}

impl_action!(Backspace, "Backspace", ActionDefinition::Backspace);

#[derive(Debug, Clone)]
pub struct InsertNewLine;

#[async_trait(?Send)]
impl Executable for InsertNewLine {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
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
        ctx.cursor.set_point(new_point, &document.buffer);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        ctx.search_buffer.reset();
        Ok(())
    }
}

impl_action!(
    InsertNewLine,
    "Insert new line",
    ActionDefinition::InsertNewLine
);

#[derive(Debug, Clone)]
pub struct InsertNewLineBelow;

#[async_trait(?Send)]
impl Executable for InsertNewLineBelow {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
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
        let byte_position = document.buffer.cursor_position(&ctx.cursor.get_point());

        // Insert newline at end of current line followed by indentation
        let insert_text = format!("\n{}", indentation);
        document.buffer.insert_string(byte_position, &insert_text);

        // Position cursor at the start of the new line after indentation
        let mut new_point = point.clone();
        new_point.row += 1;
        new_point.column = indentation.len();

        ctx.cursor.set_point(new_point, &document.buffer);
        document
            .history
            .push(Change::insert(byte_position, insert_text, point, new_point));
        document.mark_modified();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        ctx.search_buffer.reset();
        Ok(())
    }
}

impl_action!(
    InsertNewLineBelow,
    "Insert new line below",
    ActionDefinition::InsertNewLineBelow
);

#[derive(Debug, Clone)]
pub struct InsertNewLineAbove;

#[async_trait(?Send)]
impl Executable for InsertNewLineAbove {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let document = ctx.buffer_manager.current_mut();
        let point = ctx.cursor.get_point();

        // Get the indentation of the current line
        let current_line = document.buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to beginning of current line
        let mut new_point = point.clone();
        new_point.column = 0;
        let byte_position = document.buffer.cursor_position(&new_point);
        log::info!("Point: {:?}, Byte Position: {}", new_point, byte_position);

        // Insert indentation followed by newline
        let insert_text = format!("{}\n", indentation);
        document.buffer.insert_string(byte_position, &insert_text);

        // Position cursor at end of the new line (after indentation)
        new_point.column = indentation.len();

        ctx.cursor.set_point(new_point, &document.buffer);

        document.mark_modified();
        document
            .history
            .push(Change::insert(byte_position, insert_text, point, new_point));
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        ctx.search_buffer.reset();
        Ok(())
    }
}

impl_action!(
    InsertNewLineAbove,
    "Insert new line above",
    ActionDefinition::InsertNewLineAbove
);
