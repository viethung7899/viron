use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    // completion: Option<CompletionClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    definition: Option<DefinitionClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    declaration: Option<DeclarationClientCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    general: Option<GeneralCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    dynamic_registration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    dynamic_registration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct GeneralCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    position_encoding: Option<Vec<PositionEncodingKind>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionEncodingKind {
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "utf-16")]
    Utf16,
    #[serde(rename = "utf-32")]
    Utf32,
}
