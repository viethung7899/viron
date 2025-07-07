use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    // completion: Option<CompletionClientCapabilities>,
    definition: Option<DefinitionClientCapabilities>,
    declaration: Option<DeclarationClientCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionClientCapabilities {
    dynamic_registration: Option<bool>,
    link_support: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationClientCapabilities {
    dynamic_registration: Option<bool>,
    link_support: Option<bool>,
}
