use crate::core::history::edit::Edit;
use crate::core::message::Message;
use crate::core::mode::Mode;
use crate::input::actions::definition::ActionDefinition;
use crate::input::actions::{
    impl_action, movement, system, Action, ActionContext, ActionResult, Executable,
};
use async_trait::async_trait;
use std::fmt::Debug;

pub(super) async fn after_edit(ctx: &mut ActionContext<'_>, edit: &Edit) -> ActionResult {
    let document = ctx.buffer_manager.current_mut();
    document.mark_modified();

    if let Some(syntax_engine) = document.syntax_engine.as_mut() {
        syntax_engine.apply_edit(&edit)?;
    }

    ctx.compositor
        .mark_dirty(&ctx.component_ids.editor_view_id)?;
    ctx.compositor
        .mark_dirty(&ctx.component_ids.status_line_id)?;
    ctx.search_buffer.reset();
    Ok(())
}

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
        let byte_start = buffer.cursor_position(&current_point);

        let new_position = buffer.insert_char(byte_start, self.0);
        let new_point = buffer.point_at_position(new_position);

        ctx.cursor.set_point(new_point, buffer);

        let edit = Edit::insert(
            byte_start,
            current_point,
            self.0.to_string(),
            current_point,
            new_point,
        );
        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let point = ctx.cursor.get_point();
        let byte_start = buffer.cursor_position(&point);
        if let Some((c, _)) = buffer.delete_char(byte_start) {
            // Cursor stays in place after deletion
            let edit = Edit::delete(byte_start, point, c.to_string(), point, point);
            after_edit(ctx, &edit).await?;
            ctx.buffer_manager.current_mut().history.push(edit);
        }
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
        ctx.cursor.move_left(&document.buffer, ctx.mode, true);
        if position > 0 {
            if let Some((c, new_position)) = document.buffer.delete_char(position - 1) {
                let new_point = document.buffer.point_at_position(new_position);
                let edit = Edit::delete(position - 1, point, c.to_string(), point, new_point);
                after_edit(ctx, &edit).await?;
                ctx.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let point = ctx.cursor.get_point();
        let byte_start = buffer.cursor_position(&point);
        let new_position = buffer.insert_char(byte_start, '\n');
        let new_point = buffer.point_at_position(new_position);
        ctx.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(byte_start, point, "\n".to_string(), point, new_point);
        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let point = ctx.cursor.get_point();

        // Get the indentation of the current line
        let current_line = buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to end of the current line
        ctx.cursor.move_to_line_end(&buffer, &Mode::Insert);
        let byte_start = buffer.cursor_position(&ctx.cursor.get_point());
        let start_point = buffer.point_at_position(byte_start);

        // Insert newline at end of current line followed by indentation
        let insert_text = format!("\n{}", indentation);
        buffer.insert_string(byte_start, &insert_text);

        // Position cursor at the start of the new line after indentation
        let mut new_point = point.clone();
        new_point.row += 1;
        new_point.column = indentation.len();

        ctx.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(byte_start, start_point, insert_text, point, new_point);
        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let point = ctx.cursor.get_point();

        // Get the indentation of the current line
        let current_line = buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to beginning of current line
        let mut new_point = point.clone();
        new_point.column = 0;
        let start_byte = buffer.cursor_position(&new_point);
        let start_point = buffer.point_at_position(start_byte);

        // Insert indentation followed by newline
        let insert_text = format!("{}\n", indentation);
        buffer.insert_string(start_byte, &insert_text);

        // Position cursor at end of the new line (after indentation)
        new_point.column = indentation.len();

        ctx.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(start_byte, start_point, insert_text, point, new_point);

        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
        Ok(())
    }
}

impl_action!(
    InsertNewLineAbove,
    "Insert new line above",
    ActionDefinition::InsertNewLineAbove
);

#[derive(Debug, Clone)]

pub struct DeleteCurrentLine;

#[async_trait(?Send)]
impl Executable for DeleteCurrentLine {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer_mut();
        let start_point = ctx.cursor.get_point();
        let (deleted, start_byte) = buffer.delete_line(start_point.row).unwrap();
        let edit = Edit::delete(
            start_byte,
            buffer.point_at_position(start_byte),
            deleted,
            start_point,
            start_point,
        );
        after_edit(ctx, &edit).await?;
        ctx.buffer_manager.current_mut().history.push(edit);
        Ok(())
    }
}

impl_action!(
    DeleteCurrentLine,
    "Insert new line above",
    ActionDefinition::DeleteCurrentLine
);

#[derive(Debug, Clone)]
pub struct ChangeCurrentLine;

#[async_trait(?Send)]
impl Executable for ChangeCurrentLine {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        DeleteCurrentLine.execute(ctx).await?;
        InsertNewLineAbove.execute(ctx).await?;
        Ok(())
    }
}

impl_action!(
    ChangeCurrentLine,
    "Change current line",
    ActionDefinition::ChangeCurrentLine
);

#[derive(Debug, Clone)]
pub struct Undo;

#[async_trait(?Send)]
impl Executable for Undo {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        match ctx.buffer_manager.current_mut().get_undo() {
            Ok(edit) => {
                ctx.buffer_manager.current_buffer_mut().apply_edit(&edit);
                ctx.cursor
                    .set_point(edit.point_after(), ctx.buffer_manager.current_buffer());
                let (row, column) = ctx.cursor.get_display_cursor();
                movement::GoToPosition::new(row, column)
                    .execute(ctx)
                    .await?;
                after_edit(ctx, &edit).await
            }
            Err(e) => {
                system::ShowMessage(Message::error(e.to_string()))
                    .execute(ctx)
                    .await
            }
        }
    }
}

impl_action!(Undo, "Undo", ActionDefinition::Undo);

#[derive(Debug, Clone)]
pub struct Redo;

#[async_trait(?Send)]
impl Executable for Redo {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        match ctx.buffer_manager.current_mut().get_redo() {
            Ok(edit) => {
                ctx.buffer_manager.current_buffer_mut().apply_edit(&edit);
                ctx.cursor
                    .set_point(edit.point_after(), ctx.buffer_manager.current_buffer());
                let (row, column) = ctx.cursor.get_display_cursor();
                movement::GoToPosition::new(row, column)
                    .execute(ctx)
                    .await?;
                after_edit(ctx, &edit).await
            }
            Err(e) => {
                system::ShowMessage(Message::error(e.to_string()))
                    .execute(ctx)
                    .await
            }
        }
    }
}

impl_action!(Redo, "Redo", ActionDefinition::Redo);
