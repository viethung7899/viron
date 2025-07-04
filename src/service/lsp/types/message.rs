use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowMessageParams {
    #[serde(rename = "type")]
    pub type_: MessageType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessageParams {
    #[serde(rename = "type")]
    pub type_: MessageType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, PartialOrd, PartialEq, Eq, Ord)]
#[repr(u8)]
pub enum MessageType {
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
}
