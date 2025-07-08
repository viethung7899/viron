use crate::core::message::Message;
use crate::input::actions::Action;
use crate::input::actions::{
    impl_action, system, ActionContext, ActionResult, Executable,
};
use crate::service::lsp::types::Diagnostic;
use async_trait::async_trait;
use crate::input::actions::definition::ActionDefinition;

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
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

impl UpdateDiagnostics {
    pub fn new(uri: String, diagnostics: Vec<Diagnostic>) -> Self {
        Self { uri, diagnostics }
    }
}

#[async_trait(?Send)]
impl Executable for UpdateDiagnostics {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.lsp_service
            .update_diagnostics(self.uri.clone(), self.diagnostics.clone());
        if let Some(current_uri) = ctx.buffer_manager.current().uri() {
            if current_uri == self.uri {
                ctx.compositor
                    .mark_dirty(&ctx.component_ids.buffer_view_id)?;
            }
        }
        Ok(())
    }
}
