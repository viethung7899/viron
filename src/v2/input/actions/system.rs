use crate::input::actions::{
    Action, ActionContext, ActionDefinition, ActionImpl, ActionResult, impl_action,
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

impl ActionImpl for QuitEditor {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        // Access to the editor's running state
        *ctx.running = false;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::Quit { force: self.force }
    }
}

impl_action!(QuitEditor, "Quit the editor");
