use serde::{Deserialize, Serialize, Serializer};
use crate::input::keys::encode::KeyEncoder;
use crate::input::keys::KeyEvent;
use crate::input::keys::parser::parse_key_sequence;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<KeyEvent>,
}

impl KeySequence {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
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

impl Serialize for KeySequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = self.encode()
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&string)
    }
}

impl<'de> Deserialize<'de> for KeySequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_key_sequence(&s).map_err(serde::de::Error::custom)
    }
}