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

fn encode_key_code(code: KeyCode) -> Option<String> {
    let encoded = match code {
        KeyCode::Backspace => "<Backspace>".to_string(),
        KeyCode::Enter => "<Enter>".to_string(),
        KeyCode::Left => "<Left>".to_string(),
        KeyCode::Right => "<Right>".to_string(),
        KeyCode::Up => "<Up>".to_string(),
        KeyCode::Down => "<Down>".to_string(),
        KeyCode::Home => "<Home>".to_string(),
        KeyCode::End => "<End>".to_string(),
        KeyCode::PageUp => "<PageUp>".to_string(),
        KeyCode::PageDown => "<PageDown>".to_string(),
        KeyCode::Tab => "<Tab>".to_string(),
        KeyCode::BackTab => "<BackTab>".to_string(),
        KeyCode::Delete => "<Delete>".to_string(),
        KeyCode::Insert => "<Insert>".to_string(),
        KeyCode::F(n) => format!("<F{n}>"),
        KeyCode::Char(c) => {
            if c == '<' {
                "<lt>".to_string()
            } else if c == '>' {
                "<gt>".to_string()
            } else {
                c.to_string()
            }
        }
        KeyCode::Null => "<Null>".to_string(),
        KeyCode::Esc => "<Esc>".to_string(),
        KeyCode::CapsLock => "<CapsLock>".to_string(),
        KeyCode::ScrollLock => "<ScrollLock>".to_string(),
        KeyCode::NumLock => "<NumLock>".to_string(),
        KeyCode::PrintScreen => "<PrintScreen>".to_string(),
        _ => {
            return None;
        }
    };
    Some(encoded)
}