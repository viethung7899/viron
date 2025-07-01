use crate::editor::Mode;
use crate::input::actions::{impl_action, mode, Action, Executable};
use crate::input::actions::{ActionContext, ActionDefinition, ActionResult};

#[derive(Debug, Clone)]
pub struct SearchMoveLeft;

impl Executable for SearchMoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.search_buffer.buffer.move_cursor_left();
        Ok(())
    }
}

impl_action!(SearchMoveLeft, "Move cursor left in search box", self {
    ActionDefinition::SearchMoveLeft
});

#[derive(Debug, Clone)]
pub struct SearchMoveRight;

impl Executable for SearchMoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.search_buffer.buffer.move_cursor_right();
        Ok(())
    }
}

impl_action!(SearchMoveRight, "Move cursor right in search box", self {
    ActionDefinition::SearchMoveRight
});

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

impl_action!(SearchInsertChar, "Insert character in search box", self {
    ActionDefinition::SearchInsertChar { ch: self.ch }
});

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

impl_action!(SearchDeleteChar, "Delete character in search box", self {
    ActionDefinition::SearchDeleteChar
});

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
