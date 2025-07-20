use crate::actions::ActionResult;
use crate::actions::context::ActionContext;
use crate::actions::core::{ActionDefinition, Executable, impl_action};
use crate::core::message::Message;
use async_trait::async_trait;
use crate::constants::components::MESSAGE_AREA;

#[derive(Debug, Clone)]
pub struct Quit;

#[async_trait(?Send)]
impl Executable for Quit {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        // Access to the editor's running state
        *ctx.running = false;
        Ok(())
    }
}

impl_action!(Quit, "Quit the editor", ActionDefinition::Quit);

#[derive(Debug, Clone)]
pub struct ShowMessage(pub Message);

#[async_trait(?Send)]
impl Executable for ShowMessage {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.message.show_message(self.0.clone());
        ctx.ui
            .compositor
            .mark_visible(MESSAGE_AREA, true)?;
        Ok(())
    }
}
