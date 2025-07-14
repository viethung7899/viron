use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    Delete,
    Change,
}

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Delete => "d".to_string(),
            Operator::Change => "c".to_string(),
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Operator::Delete => "delete",
            Operator::Change => "change",
        }
    }
}
