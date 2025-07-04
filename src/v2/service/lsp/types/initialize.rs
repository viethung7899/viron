use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    process_id: Option<usize>,
    // client_info: Option<ClientInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initialization_options: Option<serde_json::Value>,
    capabilities: ClientCapabilities,
    // trace: Option<TraceValue>,
    // workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    // workspace: Option<WorkspaceClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_document: Option<TextDocumentClientCapabilities>,
    // window: Option<WindowClientCapabilities>,
    // experimental: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    completion: Option<super::completion::CompletionClientCapabilities>,
}
