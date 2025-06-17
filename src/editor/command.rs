use crate::editor::{Action, Mode};

#[derive(Default)]
pub struct CommandCenter {
    pub(super) command: String,
    pub(super) position: usize,
}

impl CommandCenter {
    pub fn reset(&mut self) {
        self.command.clear();
        self.position = 0;
    }

    pub fn move_left(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.position = self.command.len().min(self.position + 1);
    }

    pub fn insert(&mut self, c: char) {
        self.command.insert(self.position, c);
        self.position += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.command.is_empty() {
            false
        } else {
            self.command.remove(self.position);
            self.position = self.command.len().min(self.position);
            !self.command.is_empty()
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.position == 0 && !self.command.is_empty() {
            return true;
        }
        self.move_left();
        self.delete_char()
    }

    pub fn parse_action(&self) -> Result<Action, String> {
        if let Some(command) = self.command.strip_prefix(":") {
            return parse_command_action(command);
        }
        if let Some(term) = self.command.strip_prefix("/") {
            return Ok(Action::Find(term.to_string()));
        }
        Err(format!("Invalid command: {}", self.command))
    }
}

fn parse_command_action(command: &str) -> Result<Action, String> {
    match command {
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
