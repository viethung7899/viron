use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegisterName {
    #[default]
    Unnamed,
    Numbered(u8),
    SmallDelete
}

impl RegisterName {
    pub const LAST_YANK: RegisterName = RegisterName::Numbered(0);
    pub fn to_char(&self) -> char {
        match *self {
            RegisterName::Unnamed => '"',
            RegisterName::Numbered(number) => (number + b'0') as char,
            RegisterName::SmallDelete => '_',
        }
    }

    pub fn from_char(c: char) -> Result<RegisterName> {
        let register = match c {
            '"' => RegisterName::Unnamed,
            '0'..='9' => RegisterName::Numbered(c as u8 - b'0'),
            '_' => RegisterName::SmallDelete,
            _ => return {
                Err(anyhow!("Invalid register name: {c}"))
            }
        };
        Ok(register)
    }
    
    pub fn is_valid_name(c: char) -> bool {
        Self::from_char(c).is_ok()
    }
    
    pub fn all_names() -> Vec<RegisterName> {
        let mut registers = Vec::new();
        registers.push(RegisterName::Unnamed);
        for i in 0..=9 {
            registers.push(RegisterName::Numbered(i));
        }
        registers.push(RegisterName::SmallDelete);
        registers
    }
}