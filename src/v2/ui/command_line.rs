use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{PrintStyledContent, Stylize},
};
use std::io::Write;

pub struct CommandLine {
    command: String,
    cursor_position: usize,
    width: usize,
}

impl CommandLine {
    pub fn new(width: usize) -> Self {
        Self {
            command: String::new(),
            cursor_position: 0,
            width,
        }
    }

    pub fn render<W: Write>(&self, writer: &mut W, screen_height: usize) -> Result<()> {
        // Position cursor at bottom line
        let row = screen_height - 1;
        queue!(writer, cursor::MoveTo(0, row as u16))?;

        // Get visible part of command
        let visible = if self.command.len() > self.width {
            &self.command[self.command.len() - self.width..]
        } else {
            &self.command
        };

        // Render command
        queue!(
            writer,
            PrintStyledContent(format!(":{}", visible).stylize())
        )?;

        // Position cursor
        let cursor_x = if self.cursor_position >= self.width {
            self.width as u16
        } else {
            (self.cursor_position + 1) as u16 // +1 for the ":"
        };

        queue!(writer, cursor::MoveTo(cursor_x, row as u16))?;

        Ok(())
    }

    pub fn set_command(&mut self, command: &str) {
        self.command = command.to_string();
        self.cursor_position = self.command.len();
    }

    pub fn clear(&mut self) {
        self.command.clear();
        self.cursor_position = 0;
    }

    pub fn insert(&mut self, c: char) {
        self.command.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.command.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }
}
