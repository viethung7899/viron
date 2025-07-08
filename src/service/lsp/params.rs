use crate::service::lsp::types::capabilities::*;
use crate::service::lsp::types::initialize::*;

fn get_workspace() -> WorkspaceFolder {
    let workspace = std::env::current_dir().unwrap_or_default();
    let workspace_uri = format!("file://{}", workspace.to_string_lossy());
    let workspace_name = workspace
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Workspace")
        .to_string();
    WorkspaceFolder {
        uri: workspace_uri,
        name: workspace_name,
    }
}

pub fn get_initialize_params() -> InitializeParams {
    let client_capabilities = ClientCapabilities::builder()
        .text_document(
            TextDocumentClientCapabilities::builder()
                .definition(
                    DefinitionClientCapabilities::builder()
                        .dynamic_registration(true)
                        .link_support(false)
                        .build(),
                )
                .general(
                    GeneralCapabilities::builder()
                        .position_encoding(vec![PositionEncodingKind::Utf8])
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
        .capabilities(client_capabilities)
        .workspace_folders(vec![get_workspace()])
        .build()
}
