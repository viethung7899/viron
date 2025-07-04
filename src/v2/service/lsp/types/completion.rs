use serde::{Deserialize, Serialize};

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
