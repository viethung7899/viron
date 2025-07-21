use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use crate::input::keys::{KeyEvent};
use crate::input::keys::sequence::KeySequence;

pub trait KeyEncoder {
    fn encode(&self) -> Result<String>;
}

impl KeyEncoder for KeyCode {
    fn encode(&self) -> Result<String> {
        let encoded = match self {
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
            KeyCode::Delete => "<Delete>".to_string(),
            KeyCode::Esc => "<Esc>".to_string(),
            KeyCode::Char(c) => {
                if *c == '<' {
                    "<lt>".to_string()
                } else if *c == '>' {
                    "<gt>".to_string()
                } else {
                    c.to_string()
                }
            }
            _ => {
                return Err(anyhow!("Unsupported key code: {:?}", self));
            }
        };
        Ok(encoded)
    }
}

impl KeyEncoder for KeyEvent {
    fn encode(&self) -> Result<String> {
        let key = self.code.encode()?;
        match self.modifiers { 
            KeyModifiers::NONE => Ok(key),
            KeyModifiers::CONTROL => Ok(format!("<C-{}>", key)),
            KeyModifiers::ALT => Ok(format!("<A-{}>", key)),
            KeyModifiers::SHIFT => match self.code {
                KeyCode::Char(_) => {
                    Ok(key)
                }
                _ => Ok(format!("<S-{}>", key)),
            }
            _ => {
                Err(anyhow!("Unsupported key modifiers: {:?}", self.modifiers))
            }
        }
    }
}

impl KeyEncoder for KeySequence {
    fn encode(&self) -> Result<String> {
        let mut encoded_keys = String::new();
        for key in &self.keys {
            let encoded = key.encode()?;
            encoded_keys.push_str(&encoded);
        }
        Ok(encoded_keys)
    }
}