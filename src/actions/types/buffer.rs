use crate::actions::core::{impl_action, ActionDefinition, Executable};
use crate::actions::types::system;
use crate::actions::ActionResult;
use crate::core::message::Message;
use async_trait::async_trait;
use std::fmt::Debug;
use std::path::PathBuf;
use crate::actions::context::ActionContext;
use crate::constants::components::EDITOR_VIEW;
use crate::core::register::RegisterName;

async fn after_buffer_change(ctx: &mut ActionContext<'_>) -> ActionResult {
    let document = ctx.editor.buffer_manager.current();
    let language = document.language;

    // Update syntax highlighter with the current document's language
    if let Some(client) = ctx.lsp_service.start_server(language).await? {
        client.did_open(&document).await?;
    };

    ctx.ui.compositor.mark_all_dirty();
    Ok(())
}

#[derive(Debug, Clone)]
pub struct NextBuffer;

#[async_trait(?Send)]
impl Executable for NextBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.buffer_manager.next_buffer();
        after_buffer_change(ctx).await
    }
}

impl_action!(NextBuffer, "Next buffer", ActionDefinition::NextBuffer);

#[derive(Debug, Clone)]
pub struct PreviousBuffer;

#[async_trait(?Send)]
impl Executable for PreviousBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.buffer_manager.previous_buffer();
        after_buffer_change(ctx).await
    }
}

impl_action!(
    PreviousBuffer,
    "Previous buffer",
    ActionDefinition::PreviousBuffer
);

#[derive(Debug, Clone)]
pub struct OpenBuffer {
    path: PathBuf,
}

impl OpenBuffer {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait(?Send)]
impl Executable for OpenBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.buffer_manager.open_file(&self.path);
        after_buffer_change(ctx).await
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

#[async_trait(?Send)]
impl Executable for WriteBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let document = ctx.editor.buffer_manager.current();
        let path = self.path.clone().or(document.path.clone());
        let Some(path) = path else {
            return system::ShowMessage(Message::error(
                "No path specified for writing the buffer. Please provide a valid path."
                    .to_string(),
            ))
            .execute(ctx)
            .await;
        };

        let content = document.buffer.to_string();
        let line_count = document.buffer.line_count();

        if let Some(client) = ctx.lsp_service.get_client_mut() {
            client.did_save(document).await?;
        }

        match std::fs::write(&path, &content) {
            Ok(_) => {
                let message = format!(
                    "{:?} {}L, {}B written",
                    path.to_string_lossy().to_string(),
                    line_count,
                    content.len()
                );
                ctx.editor.buffer_manager.current_mut().modified = false;
                system::ShowMessage(Message::info(message))
                    .execute(ctx)
                    .await
            }
            Err(e) => {
                system::ShowMessage(Message::error(format!("E: {e}")))
                    .execute(ctx)
                    .await
            }
        }
    }
}

impl_action!(WriteBuffer, "Write buffer", self {
    ActionDefinition::WriteBuffer {
        path: self.path.as_ref().map(|p| p.to_string_lossy().to_string()),
    }
});

#[derive(Debug, Clone)]
pub struct CloseBuffer {
    force: bool,
}

impl CloseBuffer {
    pub fn force(force: bool) -> Self {
        Self { force }
    }
}

#[async_trait(?Send)]
impl Executable for CloseBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        if !self.force && ctx.editor.buffer_manager.current().modified {
            return system::ShowMessage(Message::error(
                "Buffer has unsaved changes. Use 'force' to close anyway.".to_string(),
            ))
            .execute(ctx)
            .await;
        }

        let document = ctx.editor.buffer_manager.close_current();
        if let Some(client) = ctx.lsp_service.get_client_mut() {
            client.did_close(&document).await?;
        }

        if ctx.editor.buffer_manager.is_empty() {
            *ctx.running = false;
        } else {
            after_buffer_change(ctx).await?;
        }
        Ok(())
    }
}

impl_action!(CloseBuffer, "Close the current buffer", self {
    ActionDefinition::CloseBuffer { force: self.force }
});

#[derive(Debug, Clone)]
pub struct RefreshBuffer;

#[async_trait(?Send)]
impl Executable for RefreshBuffer {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.ui.compositor
            .mark_dirty(EDITOR_VIEW)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SetRegister {
    name: RegisterName,
}

impl SetRegister {
    pub fn new(name: RegisterName) -> Self {
        Self { name }
    }
}

#[async_trait(?Send)]
impl Executable for SetRegister {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.editor.register_system.set_current_target(self.name);
        Ok(())
    }
}