use crate::core::message::Message;
use crate::input::actions::{
    impl_action, Action, ActionContext, ActionResult, Executable,
};
use async_trait::async_trait;
use crate::input::actions::definition::ActionDefinition;

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
        ctx.compositor
            .mark_visible(&ctx.component_ids.message_area_id, true)?;
        Ok(())
    }
}
