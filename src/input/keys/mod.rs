pub mod sequence;
mod encode;
mod parser;

use crossterm::event::{KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers};

pub use encode::KeyEncoder;

// Wrapper around crossterm's KeyEvent for easier handling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}