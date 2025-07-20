use crate::actions::ActionResult;
use crate::actions::context::ActionContext;
use crate::actions::core::{ActionDefinition, Executable, impl_action};
use crate::actions::types::{mode, movement, system};
use crate::constants::components::SEARCH_BOX;
use crate::core::message::Message;
use crate::core::mode::Mode;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct SearchMoveLeft;

#[async_trait(?Send)]
impl Executable for SearchMoveLeft {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.search_buffer.buffer.move_cursor_left();
        Ok(())
    }
}

impl_action!(
    SearchMoveLeft,
    "Move cursor left",
    ActionDefinition::SearchMoveLeft
);

#[derive(Debug, Clone)]
pub struct SearchMoveRight;

#[async_trait(?Send)]
impl Executable for SearchMoveRight {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.search_buffer.buffer.move_cursor_right();
        Ok(())
    }
}

impl_action!(
    SearchMoveRight,
    "Move cursor right",
    ActionDefinition::SearchMoveRight
);

#[derive(Debug, Clone)]
pub struct SearchInsertChar {
    ch: char,
}

impl SearchInsertChar {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

#[async_trait(?Send)]
impl Executable for SearchInsertChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.search_buffer.buffer.insert_char(self.ch);
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchDeleteChar;

#[async_trait(?Send)]
impl Executable for SearchDeleteChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.input.search_buffer.buffer.delete_char() {
            mode::EnterMode::new(Mode::Normal).execute(ctx).await?;
        }
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

impl_action!(
    SearchDeleteChar,
    "Delete character in search box",
    ActionDefinition::SearchDeleteChar
);

#[derive(Debug, Clone)]
pub struct SearchBackspace;
#[async_trait(?Send)]
impl Executable for SearchBackspace {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.input.search_buffer.buffer.backspace() {
            mode::EnterMode::new(Mode::Normal).execute(ctx).await?;
        }
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

impl_action!(
    SearchBackspace,
    "Backspace in search box",
    ActionDefinition::SearchBackspace
);

#[derive(Debug, Clone)]
pub struct SearchSubmit;

#[async_trait(?Send)]
impl Executable for SearchSubmit {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let pattern = ctx.input.search_buffer.buffer.content();

        if pattern.is_empty() {
            return system::ShowMessage(Message::error(
                "E: Search pattern cannot be empty".to_string(),
            ))
            .execute(ctx)
            .await;
        }
        let result = ctx
            .input
            .search_buffer
            .search(&pattern, &ctx.editor.buffer_manager.current_buffer());
        if let Err(e) = result {
            system::ShowMessage(Message::error(format!("E: {e}")))
                .execute(ctx)
                .await?;
        }
        if let Some(point) = ctx
            .input
            .search_buffer
            .find_first(&ctx.editor.cursor.get_point())
        {
            movement::GoToPosition::new(point.row, point.column)
                .execute(ctx)
                .await?;
        }
        mode::EnterMode::new(Mode::Normal).execute(ctx).await?;
        ctx.ui.compositor.mark_visible(SEARCH_BOX, true)?;
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

impl_action!(
    SearchSubmit,
    "Submit search",
    ActionDefinition::SearchSubmit
);

#[derive(Debug, Clone)]
pub struct FindNext;

#[async_trait(?Send)]
impl Executable for FindNext {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if let Some(point) = ctx
            .input
            .search_buffer
            .find_next(&ctx.editor.cursor.get_point())
        {
            movement::GoToPosition::new(point.row, point.column)
                .execute(ctx)
                .await?;
        }
        ctx.ui.compositor.mark_visible(SEARCH_BOX, true)?;
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

impl_action!(FindNext, "Find next match", ActionDefinition::FindNext);

#[derive(Debug, Clone)]
pub struct FindPrevious;

#[async_trait(?Send)]
impl Executable for FindPrevious {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if let Some(point) = ctx
            .input
            .search_buffer
            .find_previous(&ctx.editor.cursor.get_point())
        {
            movement::GoToPosition::new(point.row, point.column)
                .execute(ctx)
                .await?;
        }
        ctx.ui.compositor.mark_visible(SEARCH_BOX, true)?;
        ctx.ui.compositor.mark_dirty(SEARCH_BOX)?;
        Ok(())
    }
}

impl_action!(
    FindPrevious,
    "Find previous match",
    ActionDefinition::FindPrevious
);
