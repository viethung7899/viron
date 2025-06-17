use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPublishDiagnostics {
    pub uri: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    // pub severity: Option<DiagnosticSeverity>,
    pub code: Option<DiagnosticCode>,
    // pub code_description: Option<DiagnosticCodeDescription>,
    // pub source: Option<String>,
    pub message: String,
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
    pub data: Option<Value>,
    // pub tags: Option<Vec<DiagnosticTag>>,
}

impl Diagnostic {
    pub fn is_for(&self, uri: &str) -> bool {
        let Some(ref related_infos) = self.related_information else {
            return true;
        };

        related_infos.iter().any(|ri| ri.location.uri == uri)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
    Int(usize),
    String(String),
}

impl ToString for DiagnosticCode {
    fn to_string(&self) -> String {
        match self {
            DiagnosticCode::Int(size) => size.to_string(),
            DiagnosticCode::String(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCodeDescription {
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}

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
    completion: Option<CompletionClientCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilities {
    completion_item: Option<CompletionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub snippet_support: Option<bool>,
    pub commit_characters_support: Option<bool>,
    pub documentation_format: Option<Vec<MarkupKind>>,
    pub deprecated_support: Option<bool>,
    pub preselect_support: Option<bool>,
    pub tag_support: Option<TagSupport>,
    pub insert_replace_support: Option<bool>,
    pub resolve_support: Option<CompletionResolveSupport>,
    pub insert_text_mode_support: Option<InsertTextMode>,
    pub label_details_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSupport {
    value_set: Vec<CompletionItemTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResolveSupport {
    properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertTextMode {
    value_set: Vec<InsertTextMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemTag {
    Deprecated, // export const Deprecated = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}
