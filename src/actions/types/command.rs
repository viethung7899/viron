use crate::actions::command_parser::parse_command;
use crate::actions::core::{impl_action, ActionDefinition, Executable};
use crate::actions::types::{mode, system};
use crate::actions::ActionResult;
use crate::core::message::Message;
use crate::core::mode::Mode;
use async_trait::async_trait;
use crate::actions::context::ActionContext;
use crate::constants::components::COMMAND_LINE;

#[derive(Debug, Clone)]
pub struct CommandMoveLeft;

#[async_trait(?Send)]
impl Executable for CommandMoveLeft {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.command_buffer.move_cursor_left();
        Ok(())
    }
}

impl_action!(CommandMoveLeft, "Move cursor left", ActionDefinition::CommandMoveLeft);

#[derive(Debug, Clone)]
pub struct CommandMoveRight;

#[async_trait(?Send)]
impl Executable for CommandMoveRight {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.command_buffer.move_cursor_right();
        Ok(())
    }
}

impl_action!(CommandMoveRight, "Move cursor right", ActionDefinition::CommandMoveRight);

#[derive(Debug, Clone)]
pub struct CommandInsertChar {
    ch: char,
}

impl CommandInsertChar {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

#[async_trait(?Send)]
impl Executable for CommandInsertChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.input.command_buffer.insert_char(self.ch);
        ctx.ui.compositor
            .mark_dirty(COMMAND_LINE)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandDeleteChar;

#[async_trait(?Send)]
impl Executable for CommandDeleteChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.input.command_buffer.delete_char() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;
        }
        ctx.ui.compositor
            .mark_dirty(COMMAND_LINE)?;
        Ok(())
    }
}

impl_action!(CommandDeleteChar, "Command delete char", ActionDefinition::CommandDeleteChar);

#[derive(Debug, Clone)]
pub struct CommandBackspace;

#[async_trait(?Send)]
impl Executable for CommandBackspace {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.input.command_buffer.backspace() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;
        }
        ctx.ui.compositor
            .mark_dirty(COMMAND_LINE)?;
        Ok(())
    }
}

impl_action!(CommandBackspace, "Command backspace", ActionDefinition::CommandBackspace);

#[derive(Debug, Clone)]
pub struct CommandExecute;

#[async_trait(?Send)]
impl Executable for CommandExecute {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let input = ctx.input.command_buffer.content();
        Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;

        match parse_command(&input) {
            Ok(action) => match action.as_ref().execute(ctx).await {
                Ok(_) => {
                    ctx.input.command_buffer.clear();
                    ctx.ui.compositor
                        .mark_visible(COMMAND_LINE, false)?;
                }
                Err(err) => {
                    system::ShowMessage(Message::error(format!("E: {err}")))
                        .execute(ctx)
                        .await?;
                }
            },
            Err(err) => {
                system::ShowMessage(Message::error(format!("E: {err}")))
                    .execute(ctx)
                    .await?;
            }
        }

        Ok(())
    }
}

impl_action!(CommandExecute, "Execute command", ActionDefinition::CommandExecute);
