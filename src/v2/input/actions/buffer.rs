use crate::input::actions::{
    impl_action, Action, ActionContext, ActionDefinition, ActionResult,
    Executable,
};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NextBuffer;

impl Executable for NextBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.next_buffer();
        Ok(())
    }
}

impl_action!(NextBuffer, "Next buffer", self {
    ActionDefinition::NextBuffer
});

#[derive(Debug, Clone)]
pub struct PreviousBuffer;

impl Executable for PreviousBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.previous_buffer();
        Ok(())
    }
}

impl_action!(PreviousBuffer, "Previous buffer", self {
    ActionDefinition::PreviousBuffer
});

#[derive(Debug, Clone)]
pub struct OpenBuffer {
    path: PathBuf,
}

impl OpenBuffer {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Executable for OpenBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.open_file(&self.path)?;
        Ok(())
    }
}

impl_action!(OpenBuffer, "Open buffer", self {
    ActionDefinition::OpenBuffer {
        path: self.path.to_string_lossy().to_string(),
    }
});

#[derive(Debug, Clone)]
pub struct WriteBuffer {
    path: Option<PathBuf>,
}

impl WriteBuffer {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

impl Executable for WriteBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let current_document = ctx.buffer_manager.current();
        let path = self.path.clone().or(current_document.path.clone());
        let Some(path) = path else {
            return Err(anyhow::anyhow!("No path specified for writing buffer"));
        };

        let content = current_document.buffer.to_bytes();
        std::fs::write(path, content)?;

        Ok(())
    }
}

impl_action!(WriteBuffer, "Write buffer", self {
    ActionDefinition::WriteBuffer {
        path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
    }
});
