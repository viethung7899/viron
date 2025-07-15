use anyhow::Result;
use lsp_types::{
    ClientCapabilities, ClientInfo, GotoCapability, InitializeParams,
    TextDocumentClientCapabilities, Uri, WorkspaceFolder,
};
use std::str::FromStr;

fn get_workspace() -> Result<WorkspaceFolder> {
    let workspace = std::env::current_dir()?;
    let workspace_uri = format!("file://{}", workspace.to_string_lossy());
    let workspace_name = workspace
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Workspace")
        .to_string();
    Ok(WorkspaceFolder {
        uri: Uri::from_str(&workspace_uri)?,
        name: workspace_name,
    })
}

pub fn get_initialize_params() -> Result<InitializeParams> {
    let client_capabilities = ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            definition: Some(GotoCapability {
                link_support: Some(false),
                dynamic_registration: Some(true),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    Ok(InitializeParams {
        process_id: Some(std::process::id()),
        client_info: Some(ClientInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        capabilities: client_capabilities,
        workspace_folders: Some(vec![get_workspace()?]),
        ..Default::default()
    })
}
