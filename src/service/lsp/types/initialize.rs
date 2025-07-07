use super::capabilities::*;
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    process_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_info: Option<ClientInfo>,
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
    workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

impl ClientInfo {
    pub fn new(name: impl ToString, version: Option<impl ToString>) -> Self {
        let name = name.to_string();
        let version = version.map(|v| v.to_string());
        Self { name, version }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    // workspace: Option<WorkspaceClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_document: Option<TextDocumentClientCapabilities>,
    // window: Option<WindowClientCapabilities>,
    // experimental: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}
