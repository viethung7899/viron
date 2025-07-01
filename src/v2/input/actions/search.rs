use crate::editor::Mode;
use crate::input::actions::{mode, Executable};
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

impl_action!(SearchBackspace, "Backspace in search box", self {
    ActionDefinition::SearchBackspace
});
