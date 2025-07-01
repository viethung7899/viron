use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct MoveLeft {
    count: usize,
}

impl MoveLeft {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Executable for MoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_left(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveLeft, "Move cursor left", self {
    ActionDefinition::MoveLeft { count: self.count }
});

#[derive(Debug, Clone)]
pub struct MoveRight {
    count: usize,
}

impl MoveRight {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Executable for MoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_right(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveRight, "Move cursor right", self {
    ActionDefinition::MoveRight { count: self.count }
});

#[derive(Debug, Clone)]
pub struct MoveUp {
    count: usize,
}

impl MoveUp {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Executable for MoveUp {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_up(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveUp, "Move cursor up", self {
    ActionDefinition::MoveUp { count: self.count }
});

#[derive(Debug, Clone)]
pub struct MoveDown {
    count: usize,
}

impl MoveDown {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Executable for MoveDown {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        for _ in 0..self.count {
            ctx.cursor
                .move_down(ctx.buffer_manager.current_buffer(), ctx.mode);
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveDown, "Move cursor down", self {
    ActionDefinition::MoveDown { count: self.count }
});

#[derive(Debug, Clone)]
pub struct MoveToLineStart;

impl Executable for MoveToLineStart {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_line_start();
        ctx.cursor
            .find_next_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToLineStart, "Move to line start", self {
    ActionDefinition::MoveToLineStart
});

#[derive(Debug, Clone)]
pub struct MoveToLineEnd;

impl Executable for MoveToLineEnd {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .move_to_line_end(ctx.buffer_manager.current_buffer(), ctx.mode);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToLineEnd, "Move to line end", self {
    ActionDefinition::MoveToLineEnd
});

#[derive(Debug, Clone)]
pub struct MoveToTop;

impl Executable for MoveToTop {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor.move_to_top();
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToTop, "Move to top of buffer", self {
    ActionDefinition::MoveToTop
});

#[derive(Debug, Clone)]
pub struct MoveToBottom;

impl Executable for MoveToBottom {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .move_to_bottom(ctx.buffer_manager.current_buffer(), ctx.mode);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToBottom, "Move to bottom of buffer", self {
    ActionDefinition::MoveToBottom
});

#[derive(Debug, Clone)]
pub struct MoveToViewportCenter;

impl Executable for MoveToViewportCenter {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.viewport.center_on_line(
            ctx.cursor.get_position().row,
            ctx.buffer_manager.current_buffer(),
        );
        ctx.compositor
            .mark_dirty(&ctx.component_ids.buffer_view_id)?;
        ctx.compositor.mark_dirty(&ctx.component_ids.gutter_id)?;
        Ok(())
    }
}

impl_action!(MoveToViewportCenter, "Move viewport to center of buffer", self {
    ActionDefinition::MoveToViewportCenter
});

#[derive(Debug, Clone)]
pub struct MoveToNextWord;

impl Executable for MoveToNextWord {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .find_next_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToNextWord, "Move to next word", self {
    ActionDefinition::MoveToNextWord
});

#[derive(Debug, Clone)]
pub struct MoveToPreviousWord;

impl Executable for MoveToPreviousWord {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.cursor
            .find_previous_word(ctx.buffer_manager.current_buffer());
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(MoveToPreviousWord, "Move to previous word", self {
    ActionDefinition::MoveToPreviousWord
});

#[derive(Debug, Clone)]
pub struct GoToLine {
    line_number: usize,
}

impl GoToLine {
    pub fn new(line_number: usize) -> Self {
        Self { line_number }
    }
}

impl Executable for GoToLine {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let buffer = ctx.buffer_manager.current_buffer();
        ctx.cursor.go_to_line(self.line_number, buffer, ctx.mode);
        let new_line = ctx.cursor.get_position().row;
        let viewport = &ctx.viewport;
        if new_line < viewport.top_line() || new_line >= viewport.top_line() + viewport.height() {
            Action::execute(&MoveToViewportCenter, ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.status_line_id)?;
        Ok(())
    }
}

impl_action!(GoToLine, "Go to line", self {
    ActionDefinition::GoToLine { line_number: self.line_number }
});
