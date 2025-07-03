use crate::core::message::Message;
use crate::editor::Mode;
use crate::input::actions::{
    impl_action, mode, movement, system, Action, ActionDefinition, Executable,
};
use crate::input::actions::{ActionContext, ActionResult};

#[derive(Debug, Clone)]
pub struct SearchMoveLeft;

impl Executable for SearchMoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.search_buffer.buffer.move_cursor_left();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchMoveRight;

impl Executable for SearchMoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.search_buffer.buffer.move_cursor_right();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchInsertChar {
    ch: char,
}

impl SearchInsertChar {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

impl Executable for SearchInsertChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.search_buffer.buffer.insert_char(self.ch);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchDeleteChar;

impl Executable for SearchDeleteChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.search_buffer.buffer.delete_char() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchBackspace;
impl Executable for SearchBackspace {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.search_buffer.buffer.backspace() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchSubmit {
    pub pattern: String,
}

impl SearchSubmit {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

impl Executable for SearchSubmit {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if self.pattern.is_empty() {
            return system::ShowMessage(Message::error(
                "E: Search pattern cannot be empty".to_string(),
            ))
            .execute(ctx);
        }
        let result = ctx
            .search_buffer
            .search(&self.pattern, &ctx.buffer_manager.current_buffer());
        if let Err(e) = result {
            system::ShowMessage(Message::error(format!("E: {e}"))).execute(ctx)?;
        }
        if let Some(point) = ctx.search_buffer.find_first(&ctx.cursor.get_point()) {
            movement::GoToPosition::new(point.row, point.column).execute(ctx)?;
        }
        Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        ctx.compositor
            .mark_visible(&ctx.component_ids.search_box_id, true)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FindNext;

impl Executable for FindNext {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if let Some(point) = ctx.search_buffer.find_next(&ctx.cursor.get_point()) {
            movement::GoToPosition::new(point.row, point.column).execute(ctx)?;
        }
        ctx.compositor
            .mark_visible(&ctx.component_ids.search_box_id, true)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

impl_action!(FindNext, "Find next match", self {
    ActionDefinition::FindNext
});

#[derive(Debug, Clone)]
pub struct FindPrevious;

impl Executable for FindPrevious {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if let Some(point) = ctx.search_buffer.find_previous(&ctx.cursor.get_point()) {
            movement::GoToPosition::new(point.row, point.column).execute(ctx)?;
        }
        ctx.compositor
            .mark_visible(&ctx.component_ids.search_box_id, true)?;
        ctx.compositor
            .mark_dirty(&ctx.component_ids.search_box_id)?;
        Ok(())
    }
}

impl_action!(FindPrevious, "Find previous match", self {
    ActionDefinition::FindPrevious
});
