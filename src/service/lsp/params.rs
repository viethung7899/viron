use crate::service::lsp::types::capabilities::*;
use crate::service::lsp::types::initialize::*;

pub fn get_initialize_params(workspace_uri: impl ToString) -> InitializeParams {
    let workspace_uri = workspace_uri.to_string();
    let client_capabilities = ClientCapabilities::builder()
        .text_document(
            TextDocumentClientCapabilities::builder()
                .definition(
                    DefinitionClientCapabilities::builder()
                        .dynamic_registration(true)
                        .link_support(false)
                        .build(),
                )
                .build(),
        )
        .build();

    InitializeParams::builder()
        .process_id(std::process::id() as usize)
        .client_info(ClientInfo::new(
            env!("CARGO_PKG_NAME").to_string(),
            Some(env!("CARGO_PKG_VERSION").to_string()),
        ))
        .root_uri(workspace_uri.clone())
        .capabilities(client_capabilities)
        .workspace_folders(vec![WorkspaceFolder {
            uri: workspace_uri,
            name: "Workspace".to_string(),
        }])
        .build()
}
