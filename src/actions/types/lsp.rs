use crate::actions::core::{ActionDefinition, Executable, impl_action};
use crate::actions::types::system;
use crate::actions::{ActionContext, ActionResult};
use crate::core::message::Message;
use async_trait::async_trait;
use lsp_types::Diagnostic;

#[derive(Debug, Clone)]
pub struct GoToDefinition;

#[async_trait(?Send)]
impl Executable for GoToDefinition {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let Some(lsp) = ctx.lsp_service.get_client_mut() else {
            return system::ShowMessage(Message::error("LSP client is not available".to_string()))
                .execute(ctx)
                .await;
        };

        let document = ctx.buffer_manager.current();
        let point = ctx.cursor.get_point();
        if let Err(err) = lsp.goto_definition(document, point.row, point.column).await {
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
    pub uri: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl UpdateDiagnostics {
    pub fn new(uri: Option<String>, diagnostics: Vec<Diagnostic>) -> Self {
        Self { uri, diagnostics }
    }
}

#[async_trait(?Send)]
impl Executable for UpdateDiagnostics {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let document = ctx.buffer_manager.current();
        let uri = self.uri.as_ref().cloned().or_else(|| document.get_uri());

        let Some(uri) = uri else {
            return Ok(());
        };

        ctx.lsp_service
            .update_diagnostics(&uri, self.diagnostics.clone());
        if let Some(current_uri) = document.get_uri() {
            if current_uri == uri {
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.editor_view_id)?;
            }
        }
        Ok(())
    }
}
