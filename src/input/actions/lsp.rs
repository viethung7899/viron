use crate::core::message::Message;
use crate::input::actions::Action;
use crate::input::actions::{
    impl_action, system, ActionContext, ActionDefinition, ActionResult, Executable,
};
use async_trait::async_trait;
use lsp_types::Diagnostic;

#[derive(Debug, Clone)]
pub struct GoToDefinition;

#[async_trait(?Send)]
impl Executable for GoToDefinition {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let Some(uri) = ctx.buffer_manager.current().uri() else {
            return system::ShowMessage(Message::error("File are not saved".to_string()))
                .execute(ctx)
                .await;
        };

        let Some(lsp) = ctx.lsp_service.get_client_mut() else {
            return system::ShowMessage(Message::error("LSP client is not available".to_string()))
                .execute(ctx)
                .await;
        };

        let point = ctx.cursor.get_point();
        if let Err(err) = lsp.goto_definition(&uri, point.row, point.column).await {
            return system::ShowMessage(Message::error(format!("Error: {}", err)))
                .execute(ctx)
                .await;
        }
        Ok(())
    }
}

impl_action!(
    GoToDefinition,
    "Go to definition",
    ActionDefinition::GoToDefinition
);

#[derive(Debug, Clone)]
pub struct UpdateDiagnostics {
    pub path: String,
    pub diagnostics: Vec<Diagnostic>,
}

impl UpdateDiagnostics {
    pub fn new(path: String, diagnostics: Vec<Diagnostic>) -> Self {
        Self { path, diagnostics }
    }
}

#[async_trait(?Send)]
impl Executable for UpdateDiagnostics {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.lsp_service
            .update_diagnostics(self.path.clone(), self.diagnostics.clone());
        if let Some(current_path) = ctx
            .buffer_manager
            .current()
            .full_file_path()
            .as_ref()
            .and_then(|path| path.to_str())
        {
            if current_path == &self.path {
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.editor_view_id)?;
            }
        }
        Ok(())
    }
}
