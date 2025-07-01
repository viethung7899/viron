use crate::editor::Mode;
use crate::input::actions::{impl_action, mode, Action};
use crate::input::actions::{ActionContext, ActionDefinition, ActionImpl, ActionResult};

#[derive(Debug, Clone)]
pub struct CommandMoveLeft;

impl ActionImpl for CommandMoveLeft {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_left();
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct CommandMoveRight;

impl ActionImpl for CommandMoveRight {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_right();
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct CommandInsertChar {
    ch: char,
}

impl CommandInsertChar {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

impl ActionImpl for CommandInsertChar {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.insert_char(self.ch);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::CommandInsertChar { ch: self.ch }
    }
}

#[derive(Debug, Clone)]
pub struct CommandDeleteChar;

impl ActionImpl for CommandDeleteChar {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.delete_char() {
            mode::EnterMode::new(Mode::Normal).execute(ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::CommandDeleteChar
    }
}

#[derive(Debug, Clone)]
pub struct CommandBackspace;
impl ActionImpl for CommandBackspace {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.backspace() {
            mode::EnterMode::new(Mode::Normal).execute(ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::CommandBackspace
    }
}

impl_action!(CommandMoveLeft, "Move cursor left in command line");
impl_action!(CommandMoveRight, "Move cursor right in command line");
impl_action!(CommandInsertChar, "Insert character in command line");
impl_action!(CommandDeleteChar, "Delete character in command line");
impl_action!(CommandBackspace, "Backspace in command line");
