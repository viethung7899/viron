use crate::editor::Mode;
use crate::input::actions::{impl_action, mode, Action, Executable};
use crate::input::actions::{ActionContext, ActionDefinition, ActionResult};
use crate::input::command_parser::parse_command;

#[derive(Debug, Clone)]
pub struct CommandMoveLeft;

impl Executable for CommandMoveLeft {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_left();
        Ok(())
    }
}

impl_action!(CommandMoveLeft, "Move cursor left in command line", self {
    ActionDefinition::CommandMoveLeft
});

#[derive(Debug, Clone)]
pub struct CommandMoveRight;

impl Executable for CommandMoveRight {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_right();
        Ok(())
    }
}

impl_action!(CommandMoveRight, "Move cursor right in command line", self {
    ActionDefinition::CommandMoveRight
});

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

impl_action!(CommandInsertChar, "Insert character in command line", self {
    ActionDefinition::CommandInsertChar { ch: self.ch }
});

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

impl_action!(CommandDeleteChar, "Delete character in command line", self {
    ActionDefinition::CommandDeleteChar
});

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

impl_action!(CommandBackspace, "Backspace in command line", self {
    ActionDefinition::CommandBackspace
});

#[derive(Debug, Clone)]
pub struct CommandExecute;

impl Executable for CommandExecute {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let input = ctx.command_buffer.content();
        let action = parse_command(input.trim())?;
        action.execute(ctx)?;
        Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx)?;
        Ok(())
    }
}

impl_action!(CommandExecute, "Execute command in command line", self {
    ActionDefinition::CommandExecute
});
