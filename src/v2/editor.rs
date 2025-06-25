use crossterm::cursor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

impl Mode {
    fn to_name(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command | Mode::Search => "Command",
        }
    }
    fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
            _ => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}
