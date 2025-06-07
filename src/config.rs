use crate::editor::{Action, Mode};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyAction {
    Single(Action),
    Multiple(Vec<Action>),
    Nested(HashMap<String, KeyAction>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub keys: KeyMapping,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyMapping {
    pub normal: HashMap<String, KeyAction>,
    pub insert: HashMap<String, KeyAction>,
}

impl KeyMapping {
    pub fn get_key_action(&self, event: Event, mode: &Mode) -> Option<KeyAction> {
        let mapping = match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
        };

        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                let key = match code {
                    KeyCode::Char(c) => c.to_string(),
                    _ => format!("{code:?}"),
                };
                let key = match modifiers {
                    KeyModifiers::CONTROL => format!("Ctrl+{}", key),
                    KeyModifiers::ALT => format!("Alt+{}", key),
                    _ => key,
                };
                mapping.get(&key).cloned()
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_config() {
        let toml = fs::read_to_string("config.toml").unwrap();
        let config: Config = toml::from_str(&toml).unwrap();
        println!("{config:#?}")
    }
}
