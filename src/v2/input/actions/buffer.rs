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
pub struct WriteBuffer {
    path: Option<PathBuf>,
}

impl WriteBuffer {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

impl ActionImpl for WriteBuffer {
    fn execute_impl(&self, ctx: &mut ActionContext) -> ActionResult {
        let current_document = ctx.buffer_manager.current();
        let path = self.path.clone().or(current_document.path.clone());
        let Some(path) = path else {
            return Err(anyhow::anyhow!("No path specified for writing buffer"));
        };

        let content = current_document.buffer.to_bytes();
        std::fs::write(path, content)?;

        Ok(())
    }

    fn to_serializable_impl(&self) -> ActionDefinition {
        ActionDefinition::WriteBuffer {
            path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
        }
    }
}

impl_action!(OpenBuffer, "Open buffer from file");
impl_action!(PreviousBuffer, "Previous buffer");
impl_action!(NextBuffer, "Next buffer");
impl_action!(WriteBuffer, "Write buffer to file");
