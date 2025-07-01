use crate::core::message::Message;
use crate::input::actions::{
    impl_action, system, Action, ActionContext, ActionDefinition, ActionResult, Executable,
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
            return system::ShowMessage(Message::error(
                "No path specified for writing the buffer. Please provide a valid path."
                    .to_string(),
            ))
            .execute(ctx);
        };

        let content = current_document.buffer.to_bytes();
        let line_count = current_document.buffer.line_count();
        match std::fs::write(&path, &content) {
            Ok(_) => {
                let message = format!(
                    "{:?} {}L, {}B written",
                    path.to_string_lossy().to_string(),
                    line_count,
                    content.len()
                );
                system::ShowMessage(Message::info(message)).execute(ctx)
            }
            Err(e) => system::ShowMessage(Message::error(format!("E: {e}"))).execute(ctx),
        }
    }
}

impl_action!(WriteBuffer, "Write buffer", self {
    ActionDefinition::WriteBuffer {
        path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
    }
});
