use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::common::{Location, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPublishDiagnostics {
    pub uri: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    #[serde(default)]
    pub severity: DiagnosticSeverity,
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

#[derive(
    Debug, Clone, Serialize_repr, Deserialize_repr, Default, PartialOrd, PartialEq, Eq, Ord,
)]
#[repr(usize)]
pub enum DiagnosticSeverity {
    #[default]
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
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
enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}
