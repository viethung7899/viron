use crossterm::cursor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Mode::Normal => "normal".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Command => "command".to_string(),
            Mode::Search => "search".to_string(),
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
            Mode::Search => "Search",
        }
    }

    pub fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
            _ => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}
