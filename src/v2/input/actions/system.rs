use crate::core::message::Message;
use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct QuitEditor {
    force: bool,
}

impl QuitEditor {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

#[async_trait(?Send)]
impl Executable for QuitEditor {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        // Access to the editor's running state
        *ctx.running = false;
        Ok(())
    }
}

impl_action!(QuitEditor, "Quit the editor", self {
    ActionDefinition::Quit { force: self.force }
});

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
