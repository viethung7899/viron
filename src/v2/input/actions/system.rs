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
