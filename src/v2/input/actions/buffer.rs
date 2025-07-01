use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionImpl, ActionResult,
};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NextBuffer;

impl ActionImpl for NextBuffer {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.next_buffer();
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::NextBuffer
    }
}

#[derive(Debug, Clone)]
pub struct PreviousBuffer;

impl ActionImpl for PreviousBuffer {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.previous_buffer();
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::PreviousBuffer
    }
}

#[derive(Debug, Clone)]
pub struct OpenBuffer {
    path: PathBuf,
}

impl OpenBuffer {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ActionImpl for OpenBuffer {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.open_file(&self.path)?;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::OpenBuffer {
            path: self.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuitEditor;

impl ActionImpl for QuitEditor {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        // Access to the editor's running state
        *ctx.running = false;
        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::Quit
    }
}

impl_action!(OpenBuffer, "Open buffer from file");
impl_action!(QuitEditor, "Quit the editor");
impl_action!(PreviousBuffer, "Previous buffer");
impl_action!(NextBuffer, "Next buffer");
