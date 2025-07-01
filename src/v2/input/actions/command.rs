use crate::core::message::Message;
use crate::editor::Mode;
use crate::input::actions::{mode, system, Executable};
use crate::input::actions::{ActionContext, ActionResult};
use crate::input::command_parser::parse_command;

#[derive(Debug, Clone)]
pub struct CommandMoveLeft;

impl Executable for CommandMoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_left();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandMoveRight;

impl Executable for CommandMoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_right();
        Ok(())
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

impl Executable for CommandInsertChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.insert_char(self.ch);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandDeleteChar;

impl Executable for CommandDeleteChar {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.delete_char() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandBackspace;
impl Executable for CommandBackspace {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.backspace() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandExecute;

impl Executable for CommandExecute {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let input = ctx.command_buffer.content();
        Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;

        let result =
            parse_command(&input).and_then(|action| Executable::execute(action.as_ref(), ctx));
        if let Err(err) = result {
            system::ShowMessage(Message::error(format!("E: {err}"))).execute(ctx)?;
        }

        Ok(())
    }
}
