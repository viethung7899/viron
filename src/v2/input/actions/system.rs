use crate::core::message::Message;
use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult, Executable,
};

#[derive(Debug, Clone)]
pub struct QuitEditor {
    force: bool,
}

impl QuitEditor {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

impl Executable for QuitEditor {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
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

impl Executable for ShowMessage {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.message.show_message(self.0.clone());
        ctx.compositor
            .mark_visible(&ctx.component_ids.message_area_id, true)?;
        Ok(())
    }
}
