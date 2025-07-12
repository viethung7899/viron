use crossterm::cursor;
use serde::{Deserialize, Serialize};
use crate::core::operation::Operator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    OperationPending(Operator),
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Mode::Normal => "normal".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Command => "command".to_string(),
            Mode::Search => "search".to_string(),
            Mode::OperationPending(_) => "o-pending".to_string(),
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Mode::Normal => "normal",
            Mode::Insert => "insert",
            Mode::Command => "command",
            Mode::Search => "search",
            Mode::OperationPending(_) => "o-pending",
        }
    }

    pub fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
            _ => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}
