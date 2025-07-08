use super::capabilities::*;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    process_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_info: Option<ClientInfo>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // locale: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // initialization_options: Option<serde_json::Value>,
    capabilities: ClientCapabilities,
    // trace: Option<TraceValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(default)]
    pub position_encoding: PositionEncodingKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSyncOptions {
    open_close: Option<bool>,
    #[serde(default)]
    change: TextDocumentSyncKind,
}

#[derive(Debug, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(usize)]
pub enum TextDocumentSyncKind {
    #[default]
    None = 0,
    Full = 1,
    Incremental = 2,
}
