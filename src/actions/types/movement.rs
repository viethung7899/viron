use crate::actions::core::{impl_action, ActionDefinition, Executable};
use crate::actions::ActionResult;
use crate::config::editor::Gutter;
use async_trait::async_trait;
use std::fmt::Debug;
use crate::actions::context::ActionContext;

#[derive(Debug, Clone)]
pub struct MoveLeft {
    inline: bool,
}

impl MoveLeft {
    pub fn new(inline: bool) -> Self {
        Self { inline }
    }
}

#[async_trait(?Send)]
impl Executable for MoveLeft {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let old_row = ctx.editor.cursor.get_point().row;
        ctx.editor.cursor
            .move_left(ctx.editor.buffer_manager.current_buffer(), ctx.editor.mode, self.inline);
        let new_row = ctx.editor.cursor.get_point().row;
        if old_row != new_row && ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveLeft, "Move cursor left", self {
    ActionDefinition::MoveLeft { inline: self.inline }
});

#[derive(Debug, Clone)]
pub struct MoveRight {
    inline: bool,
}

impl MoveRight {
    pub fn new(inline: bool) -> Self {
        Self { inline }
    }
}

#[async_trait(?Send)]
impl Executable for MoveRight {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let old_row = ctx.editor.cursor.get_point().row;
        ctx.editor.cursor
            .move_right(ctx.editor.buffer_manager.current_buffer(), ctx.editor.mode, self.inline);
        let new_row = ctx.editor.cursor.get_point().row;
        if old_row != new_row && ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveRight, "Move cursor right", self {
    ActionDefinition::MoveRight { inline: self.inline }
});

#[derive(Debug, Clone)]
pub struct MoveUp;

#[async_trait(?Send)]
impl Executable for MoveUp {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.cursor
            .move_up(ctx.editor.buffer_manager.current_buffer(), ctx.editor.mode);
        if ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveUp, "Move cursor up", ActionDefinition::MoveUp);

#[derive(Debug, Clone)]
pub struct MoveDown;

#[async_trait(?Send)]
impl Executable for MoveDown {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.cursor
            .move_down(ctx.editor.buffer_manager.current_buffer(), ctx.editor.mode);
        if ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveDown, "Move cursor down", ActionDefinition::MoveDown);

#[derive(Debug, Clone)]
pub struct MoveToLineStart;

#[async_trait(?Send)]
impl Executable for MoveToLineStart {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.cursor.move_to_line_start();
        ctx.editor.cursor
            .find_next_word(ctx.editor.buffer_manager.current_buffer());
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(
    MoveToLineStart,
    "Move to line start",
    ActionDefinition::MoveToLineStart
);

#[derive(Debug, Clone)]
pub struct MoveToLineEnd;

#[async_trait(?Send)]
impl Executable for MoveToLineEnd {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.cursor
            .move_to_line_end(ctx.editor.buffer_manager.current_buffer(), ctx.editor.mode);
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(
    MoveToLineEnd,
    "Move to line end",
    ActionDefinition::MoveToLineEnd
);

#[derive(Debug, Clone)]
pub struct MoveToTop;

#[async_trait(?Send)]
impl Executable for MoveToTop {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        GoToLine::new(0).execute(ctx).await
    }
}

impl_action!(
    MoveToTop,
    "Move to top of buffer",
    ActionDefinition::MoveToTop
);

#[derive(Debug, Clone)]
pub struct MoveToBottom;

#[async_trait(?Send)]
impl Executable for MoveToBottom {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let line_count = ctx.editor.buffer_manager.current_buffer().line_count();
        GoToLine::new(line_count.saturating_sub(1))
            .execute(ctx)
            .await
    }
}

impl_action!(
    MoveToBottom,
    "Move to bottom of buffer",
    ActionDefinition::MoveToBottom
);

#[derive(Debug, Clone)]
pub struct MoveToViewportCenter;

#[async_trait(?Send)]
impl Executable for MoveToViewportCenter {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.viewport.center_on_line(
            ctx.editor.cursor.get_point().row,
            ctx.editor.buffer_manager.current_buffer(),
        );
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        Ok(())
    }
}

impl_action!(
    MoveToViewportCenter,
    "Move viewport to center of buffer",
    ActionDefinition::MoveToViewportCenter
);

#[derive(Debug, Clone)]
pub struct MoveToNextWord;

#[async_trait(?Send)]
impl Executable for MoveToNextWord {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let old_row = ctx.editor.cursor.get_point().row;
        let buffer = ctx.editor.buffer_manager.current_buffer();
        let cursor = ctx.editor.cursor.find_next_word(buffer);
        if cursor.get_point().row != old_row && ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.editor.cursor.set_point(cursor.get_point(), buffer);
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(
    MoveToNextWord,
    "Move to next word",
    ActionDefinition::MoveToNextWord
);

#[derive(Debug, Clone)]
pub struct MoveToPreviousWord;

#[async_trait(?Send)]
impl Executable for MoveToPreviousWord {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let old_row = ctx.editor.cursor.get_point().row;
        let buffer = ctx.editor.buffer_manager.current_buffer();
        let cursor = ctx.editor.cursor.find_previous_word(buffer);
        if cursor.get_point().row != old_row && ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.editor.cursor.set_point(cursor.get_point(), buffer);
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(
    MoveToPreviousWord,
    "Move to previous word",
    ActionDefinition::MoveToPreviousWord
);

#[derive(Debug, Clone)]
pub struct GoToLine {
    line_number: usize,
}

impl GoToLine {
    pub fn new(line_number: usize) -> Self {
        Self { line_number }
    }
}

#[async_trait(?Send)]
impl Executable for GoToLine {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let old_line = ctx.editor.cursor.get_point().row;
        let buffer = ctx.editor.buffer_manager.current_buffer();
        ctx.editor.cursor.go_to_line(self.line_number, buffer, ctx.editor.mode);
        let new_line = ctx.editor.cursor.get_point().row;
        let viewport = &ctx.editor.viewport;
        if new_line < viewport.top_line() || new_line >= viewport.top_line() + viewport.height() {
            MoveToViewportCenter.execute(ctx).await?;
        } else if old_line != new_line && ctx.config.gutter == Gutter::Relative {
            ctx.ui.compositor
                .mark_dirty(&ctx.ui.component_ids.editor_view_id)?;
        }
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(GoToLine, "Go to line", self {
    ActionDefinition::GoToLine { line_number: self.line_number }
});

#[derive(Debug, Clone)]
pub struct GoToPosition {
    row: usize,
    column: usize,
}

impl GoToPosition {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}
#[async_trait(?Send)]
impl Executable for GoToPosition {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        GoToLine::new(self.row).execute(ctx).await?;
        let buffer = ctx.editor.buffer_manager.current_buffer();
        ctx.editor.cursor.go_to_column(self.column, buffer, ctx.editor.mode);
        ctx.ui.compositor
            .mark_dirty(&ctx.ui.component_ids.status_line_id)?;
        Ok(())
    }
}
