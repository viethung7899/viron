use crossterm::event::{KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<KeyEvent>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn from_keys(keys: Vec<KeyEvent>) -> Self {
        Self { keys }
    }

    pub fn add(&mut self, key: KeyEvent) {
        self.keys.push(key);
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn is_prefix_of(&self, other: &KeySequence) -> bool {
        if self.keys.len() > other.keys.len() {
            return false;
        }

        for (i, key) in self.keys.iter().enumerate() {
            if *key != other.keys[i] {
                return false;
            }
        }

        true
    }
}

impl KeySequence {
    pub fn to_string(&self) -> String {
        self.keys
            .iter()
            .map(|key| match key.code {
                KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => c.to_string(),
                KeyCode::Char(c) => format!("<{:?}-{}>", key.modifiers, c),
                _ => format!("<{:?}>", key.code),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        let mut keys = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                // Parse special key
                let mut special = String::new();
                while let Some(next) = chars.next() {
                    if next == '>' {
                        break;
                    }
                    special.push(next);
                }

                // Parse the special key
                if special.contains('-') {
                    let parts: Vec<&str> = special.split('-').collect();
                    let modifier_str = parts[0];
                    let key_str = parts[1];

                    let modifiers = match modifier_str {
                        "SHIFT" => KeyModifiers::SHIFT,
                        "CONTROL" => KeyModifiers::CONTROL,
                        "ALT" => KeyModifiers::ALT,
                        _ => KeyModifiers::NONE,
                    };

                    if key_str.len() == 1 {
                        let c = key_str.chars().next().unwrap();
                        keys.push(KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers,
                        });
                    }
                } else {
                    // Handle special keys like <Esc>, <Enter>, etc.
                    let code = match special.as_str() {
                        "Esc" => KeyCode::Esc,
                        "Enter" => KeyCode::Enter,
                        "Backspace" => KeyCode::Backspace,
                        "Tab" => KeyCode::Tab,
                        "Space" => KeyCode::Char(' '),
                        "Left" => KeyCode::Left,
                        "Right" => KeyCode::Right,
                        "Up" => KeyCode::Up,
                        "Down" => KeyCode::Down,
                        "Home" => KeyCode::Home,
                        "End" => KeyCode::End,
                        // Add more special keys as needed
                        _ => continue, // Skip unknown keys
                    };

                    keys.push(KeyEvent {
                        code,
                        modifiers: KeyModifiers::NONE,
                    });
                }
            } else {
                // Regular character
                keys.push(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: if c.is_ascii_uppercase() {
                        KeyModifiers::SHIFT
                    } else {
                        KeyModifiers::NONE
                    },
                });
            }
        }

        Ok(KeySequence::from_keys(keys))
    }
}
