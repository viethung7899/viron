use crate::actions::core::{impl_action, ActionDefinition, Executable};
use crate::actions::types::{movement, system};
use crate::actions::ActionResult;
use crate::core::history::edit::Edit;
use crate::core::message::Message;
use crate::core::mode::Mode;
use async_trait::async_trait;
use std::fmt::Debug;
use crossterm::ExecutableCommand;
use crate::actions::context::ActionContext;
use crate::actions::core::ActionDefinition::{MoveRight, MoveToLineEnd};
use crate::actions::movement::MoveToLineStart;
use crate::constants::components::{EDITOR_VIEW, STATUS_LINE};
use crate::core::register::RegisterType;

pub(super) async fn after_edit(
    ctx: &mut ActionContext<'_>,
    edit: &Edit,
) -> ActionResult {
    let document = ctx.editor.buffer_manager.current_mut();
    document.mark_modified();

    ctx.ui.compositor
        .mark_dirty(EDITOR_VIEW)?;
    ctx.ui.compositor
        .mark_dirty(STATUS_LINE)?;
    ctx.input.search_buffer.reset();

    if let Some(syntax_engine) = document.syntax_engine.as_mut() {
        syntax_engine.apply_edit(&edit)?;
    }

    if let Some(client) = ctx.lsp_service.get_client_mut() {
        client.did_change(document).await?;
    }
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
        let current_point = ctx.editor.cursor.get_point();

        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let byte_start = buffer.cursor_position(&current_point);

        let new_position = buffer.insert_char(byte_start, self.0);
        let new_point = buffer.point_at_position(new_position);

        ctx.editor.cursor.set_point(new_point, buffer);

        let edit = Edit::insert(
            byte_start,
            current_point,
            self.0.to_string(),
            current_point,
            new_point,
        );
        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
        Ok(())
    }
}
impl_action!(InsertChar, "Insert char", self {
    ActionDefinition::InsertChar { ch: self.0 }
});

#[derive(Debug, Clone)]
pub struct DeleteChar {
    inline: bool,
}

impl DeleteChar {
    pub fn new(inline: bool) -> Self {
        Self { inline }
    }
}

#[async_trait(?Send)]
impl Executable for DeleteChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let point = ctx.editor.cursor.get_point();
        let byte_start = buffer.cursor_position(&point);

        let Some(char) = buffer.get_char(byte_start) else {
            return Ok(());
        };

        if self.inline && char == '\n' {
            return Ok(());
        }

        if let Some((c, _)) = buffer.delete_char(byte_start) {
            ctx.editor.cursor.clamp_column(buffer, ctx.editor.mode);
            let edit = Edit::delete(
                byte_start,
                point,
                c.to_string(),
                point,
                ctx.editor.cursor.get_point(),
            );
            after_edit(ctx, &edit).await?;
            ctx.editor.buffer_manager.register_manager.on_delete(c.to_string(), RegisterType::Character);
            ctx.editor.buffer_manager.current_mut().history.push(edit);
        }
        Ok(())
    }
}

impl_action!(DeleteChar, "Delete character", self {
    ActionDefinition::DeleteChar { inline: self.inline }
});

#[derive(Debug, Clone)]
pub struct Backspace {
    inline: bool,
}

impl Backspace {
    pub fn new(inline: bool) -> Self {
        Self { inline }
    }
}

#[async_trait(?Send)]
impl Executable for Backspace {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let document = ctx.editor.buffer_manager.current_mut();
        let point = ctx.editor.cursor.get_point();

        if point.column == 0 && self.inline {
            return Ok(());
        }

        let position = document.buffer.cursor_position(&point);
        ctx.editor.cursor
            .move_left(&document.buffer, ctx.editor.mode, self.inline);
        if position > 0 {
            if let Some((c, new_position)) = document.buffer.delete_char(position - 1) {
                let new_point = document.buffer.point_at_position(new_position);
                let edit = Edit::delete(position - 1, point, c.to_string(), point, new_point);
                after_edit(ctx, &edit).await?;
                ctx.editor.buffer_manager.current_mut().history.push(edit);
            }
        }
        Ok(())
    }
}

impl_action!(Backspace, "Backspace", self {
    ActionDefinition::Backspace { inline: self.inline }
});

#[derive(Debug, Clone)]
pub struct InsertNewLine;

#[async_trait(?Send)]
impl Executable for InsertNewLine {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let point = ctx.editor.cursor.get_point();
        let byte_start = buffer.cursor_position(&point);
        let new_position = buffer.insert_char(byte_start, '\n');
        let new_point = buffer.point_at_position(new_position);
        ctx.editor.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(byte_start, point, "\n".to_string(), point, new_point);
        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let point = ctx.editor.cursor.get_point();

        // Get the indentation of the current line
        let current_line = buffer.get_content_line(point.row);
        let indentation = current_line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect::<String>();

        // Move cursor to end of the current line
        ctx.editor.cursor.move_to_line_end(&buffer, &Mode::Insert);
        let byte_start = buffer.cursor_position(&ctx.editor.cursor.get_point());
        let start_point = buffer.point_at_position(byte_start);

        // Insert newline at end of current line followed by indentation
        let insert_text = format!("\n{}", indentation);
        buffer.insert_string(byte_start, &insert_text);

        // Position cursor at the start of the new line after indentation
        let mut new_point = point.clone();
        new_point.row += 1;
        new_point.column = indentation.len();

        ctx.editor.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(byte_start, start_point, insert_text, point, new_point);
        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let point = ctx.editor.cursor.get_point();

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

        ctx.editor.cursor.set_point(new_point, &buffer);
        let edit = Edit::insert(start_byte, start_point, insert_text, point, new_point);

        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
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
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();
        let start_point = ctx.editor.cursor.get_point();
        let (deleted, start_byte) = buffer.delete_line(start_point.row).unwrap();
        let edit = Edit::delete(
            start_byte,
            buffer.point_at_position(start_byte),
            deleted.clone(),
            start_point,
            start_point,
        );
        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
        ctx.editor.buffer_manager.register_manager.on_delete(deleted, RegisterType::Line);
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
        match ctx.editor.buffer_manager.current_mut().get_undo() {
            Ok(edit) => {
                ctx.editor.buffer_manager.current_buffer_mut().apply_edit(&edit);
                ctx.editor.cursor
                    .set_point(edit.point_after(), ctx.editor.buffer_manager.current_buffer());
                let (row, column) = ctx.editor.cursor.get_display_cursor();
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
        match ctx.editor.buffer_manager.current_mut().get_redo() {
            Ok(edit) => {
                ctx.editor.buffer_manager.current_buffer_mut().apply_edit(&edit);
                ctx.editor.cursor
                    .set_point(edit.point_after(), ctx.editor.buffer_manager.current_buffer());
                let (row, column) = ctx.editor.cursor.get_display_cursor();
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

#[derive(Debug, Clone)]
pub struct Paste {
    after_cursor: bool,
}

impl Paste {
    pub fn new(after_cursor: bool) -> Self {
        Self { after_cursor }
    }
}

#[async_trait(?Send)]
impl Executable for Paste {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let Some(register) = ctx.editor.buffer_manager.register_manager.get_register('"').cloned() else {
            return Ok(());
        };

        log::info!("Paste: {:?}", register);

        if register.is_empty() {
            return Ok(());
        }

        let mut cursor = ctx.editor.cursor.clone();
        let buffer = ctx.editor.buffer_manager.current_buffer_mut();

        // Move the cursor to the correct position based on the register type
        log::info!("Before cursor: {:?}", cursor);
        match (self.after_cursor, &register.register_type) {
            (true, RegisterType::Character) => {
                cursor.move_right(buffer, &Mode::Insert, false);
            }
            (false, RegisterType::Line) => {
                cursor.move_to_line_start();
            }
            (true, RegisterType::Line) => {
                cursor.move_down(buffer, &Mode::Insert);
                cursor.move_to_line_start();
            }
            _ => {}
        }

        // Insert pasted text
        log::info!("After cursor: {:?}", cursor);
        let point = cursor.get_point();
        let byte_start = buffer.cursor_position(&point);
        let new_position = buffer.insert_string(byte_start, &register.content);

        let old_point = ctx.editor.cursor.get_point();

        match register.register_type {
            RegisterType::Character => {
                ctx.editor.cursor.set_point(
                    buffer.point_at_position(new_position),
                    buffer,
                );
            }
            RegisterType::Line => {
                ctx.editor.cursor.set_point(
                    cursor.get_point(),
                    buffer
                );
            }
        }

        let new_point = ctx.editor.cursor.get_point();

        let edit = Edit::insert(
            byte_start,
            point,
            register.content.to_string(),
            old_point,
            new_point,
        );
        after_edit(ctx, &edit).await?;
        ctx.editor.buffer_manager.current_mut().history.push(edit);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PasteBeforeCursor;

#[async_trait(?Send)]
impl Executable for PasteBeforeCursor {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        Paste::new(false).execute(ctx).await
    }
}

impl_action!(
    PasteBeforeCursor,
    "Paste before cursor",
    ActionDefinition::PasteBeforeCursor
);

#[derive(Debug, Clone)]
pub struct PasteAfterCursor;

#[async_trait(?Send)]
impl Executable for PasteAfterCursor {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        Paste::new(true).execute(ctx).await
    }
}

impl_action!(
    PasteAfterCursor,
    "Paste after cursor",
    ActionDefinition::PasteAfterCursor
);
