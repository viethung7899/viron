use crate::core::message::Message;
use crate::input::actions::Action;
use crate::input::actions::{
    impl_action, system, ActionContext, ActionDefinition, ActionResult, Executable,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct GoToDefinition;

#[async_trait(?Send)]
impl Executable for GoToDefinition {
    async fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        let Some(lsp_client) = ctx.lsp_client else {
            return system::ShowMessage(Message::error("LSP client not available".to_string()))
                .execute(ctx)
                .await;
        };
        let Some(uri) = ctx.buffer_manager.current().uri() else {
            return system::ShowMessage(Message::error("File are not saved".to_string()))
                .execute(ctx)
                .await;
        };
        let point = ctx.cursor.get_point();
        if let Err(err) = lsp_client
            .goto_definition(&uri, point.row, point.column)
            .await
        {
            system::ShowMessage(Message::error("File are not saved".to_string()))
                .execute(ctx)
                .await
        } else {
            Ok(())
        }
    }
}

impl_action!(GoToDefinition, "Go to definition", self {
    ActionDefinition::GoToDefinition
});
