use crate::editor::{Action, Mode};

#[derive(Default)]
pub struct CommandCenter {
    pub(super) buffer: String,
    pub(super) position: usize,
}

impl CommandCenter {
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }

    pub fn move_left(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.position = self.buffer.len().min(self.position + 1);
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.position, c);
        self.position += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.buffer.is_empty() {
            false
        } else {
            self.buffer.remove(self.position);
            self.position = self.buffer.len().min(self.position);
            true
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.position == 0 && !self.buffer.is_empty() {
            return true;
        }
        self.move_left();
        self.delete_char()
    }

    pub fn parse_action(&self) -> Result<Action, String> {
        match self.buffer.as_str() {
            "" => Ok(Action::EnterMode(Mode::Normal)),
            "q" => Ok(Action::Quit),
            "w" => Ok(Action::Save),
            command => {
                if let Ok(line_number) = command.parse::<usize>() {
                    Ok(Action::GotoLine(line_number.saturating_sub(1)))
                } else {
                    Err(format!("Invalid command: {}", command))
                }
            }
        }
    }
}
