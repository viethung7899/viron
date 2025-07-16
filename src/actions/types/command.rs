use crate::actions::command_parser::parse_command;
use crate::actions::core::Executable;
use crate::actions::types::{mode, system};
use crate::actions::{ActionContext, ActionResult};
use crate::core::message::Message;
use crate::core::mode::Mode;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct CommandMoveLeft;

#[async_trait(?Send)]
impl Executable for CommandMoveLeft {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.move_cursor_left();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandMoveRight;

#[async_trait(?Send)]
impl Executable for CommandMoveRight {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
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

#[async_trait(?Send)]
impl Executable for CommandInsertChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.command_buffer.insert_char(self.ch);
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandDeleteChar;

#[async_trait(?Send)]
impl Executable for CommandDeleteChar {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.delete_char() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandBackspace;

#[async_trait(?Send)]
impl Executable for CommandBackspace {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !ctx.command_buffer.backspace() {
            Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;
        }
        ctx.compositor
            .mark_dirty(&ctx.component_ids.command_line_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandExecute;

#[async_trait(?Send)]
impl Executable for CommandExecute {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let input = ctx.command_buffer.content();
        Executable::execute(&mode::EnterMode::new(Mode::Normal), ctx).await?;

        match parse_command(&input) {
            Ok(action) => match action.as_ref().execute(ctx).await {
                Ok(_) => {
                    ctx.command_buffer.clear();
                    ctx.compositor
                        .mark_visible(&ctx.component_ids.command_line_id, false)?;
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
