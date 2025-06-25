use crate::v1::editor::Action;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type KeyMapping = HashMap<String, KeyAction>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyAction {
    Single(Action),
    Multiple(Vec<Action>),
    Nested(KeyMapping),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub log_file: Option<String>,
    pub theme: String,
    pub keys: Keys,
    pub tab_size: usize,
    #[serde(default = "default_mouse_scroll_line")]
    pub mouse_scroll_lines: usize,
    #[serde(default = "default_show_diagnostics")]
    pub show_diagnostics: bool,
}

fn default_mouse_scroll_line() -> usize {
    3
}

fn default_show_diagnostics() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keys {
    pub normal: KeyMapping,
    pub insert: KeyMapping,
    pub command: KeyMapping,
}

pub fn get_key_action(mapping: &KeyMapping, event: &Event) -> Option<KeyAction> {
    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => {
            let key = match code {
                KeyCode::Char(c) => c.to_string(),
                _ => format!("{code:?}"),
            };
            let key = match *modifiers {
                KeyModifiers::CONTROL => format!("Ctrl-{}", key),
                KeyModifiers::ALT => format!("Alt-{}", key),
                _ => key,
            };
            mapping.get(&key).cloned()
        }
        _ => None,
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
