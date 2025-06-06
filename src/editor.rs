use std::io::{Stdout, Write, stdout};

use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, KeyCode, KeyModifiers},
    style::{self, Color, Stylize},
    terminal,
};

use crate::{buffer::Buffer, log};

#[derive(Debug)]
pub struct Editor {
    stdout: Stdout,
    buffer: Buffer,
    offset: Position,
    cursor: Position,
    mode: Mode,
    size: (u16, u16),
}

#[derive(Debug, Clone, Default)]
struct Position {
    row: u16,
    col: u16,
}

#[derive(Debug, Clone, Default)]
enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug)]
enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    MoveToLineStart,
    MoveToLineEnd,

    EnterMode(Mode),
    InsertChar(char),
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.flush();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Self> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(Self {
            stdout,
            buffer,
            offset: Position::default(),
            cursor: Position::default(),
            mode: Mode::default(),
            size: terminal::size()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.reset_bounds();
            self.draw()?;
            if let Some(action) = self.handle_event(event::read()?)? {
                let (viewport_width, viewport_height) = self.get_viewport_size();
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        if self.cursor.row == 0 {
                            self.offset.row = self.offset.row.saturating_sub(1);
                        } else {
                            self.cursor.row -= 1;
                        }
                    }
                    Action::MoveDown => {
                        if self.cursor.row + 1 >= viewport_height {
                            self.offset.row += 1;
                        } else {
                            self.cursor.row += 1;
                        }
                    }
                    Action::MoveLeft => {
                        self.cursor.col = self.cursor.col.saturating_sub(1);
                    }
                    Action::MoveRight => {
                        self.cursor.col += 1;
                    }
                    Action::PageUp => {
                        let (_, height) = self.get_viewport_size();
                        self.offset.row = self.offset.row.saturating_sub(height);
                    }
                    Action::PageDown => {
                        let (_, height) = self.get_viewport_size();
                        log!("Page Down {}", self.offset.row);
                        if self.buffer.len() > (self.offset.row + height) as usize {
                            self.offset.row += height;
                        }
                    }
                    Action::MoveToLineStart => {
                        self.cursor.col = 0;
                    }
                    Action::MoveToLineEnd => {
                        self.cursor.col = self
                            .get_viewport_line(self.cursor.row)
                            .map_or(0, |line| line.len() as u16)
                    }
                    Action::EnterMode(mode) => {
                        self.mode = mode;
                    }
                    Action::InsertChar(c) => {
                        self.stdout
                            .queue(cursor::MoveTo(self.cursor.col, self.cursor.row))?;
                        self.stdout.queue(style::Print(c))?;
                        self.cursor.col += 1;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_viewport_size(&self) -> (u16, u16) {
        let (width, height) = self.size;
        (width, height - 2)
    }

    fn reset_bounds(&mut self) {
        let (width, height) = self.get_viewport_size();
        let current_length = self
            .get_viewport_line(self.cursor.row)
            .map_or(0, |line| line.len()) as u16;

        let row = self.cursor.row.min(height - 1);
        let col = self
            .cursor
            .col
            .min(width - 1)
            .min(current_length.saturating_sub(1));

        self.cursor.row = row;
        self.cursor.col = col;
    }

    fn get_buffer_line(&self, line: u16) -> u16 {
        self.offset.row + line
    }

    fn get_viewport_line(&self, line: u16) -> Option<String> {
        let buffer_line = self.get_buffer_line(line) as usize;
        self.buffer.get_line(buffer_line)
    }

    fn draw(&mut self) -> Result<()> {
        self.draw_viewport()?;
        self.draw_status_line()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.col, self.cursor.row))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_viewport(&mut self) -> Result<()> {
        let (width, height) = self.get_viewport_size();
        for i in 0..height {
            let line = self.get_viewport_line(i).unwrap_or_default();
            let formatted_line = format!("{line:<w$}", w = width as usize,);
            self.stdout.queue(cursor::MoveTo(0, i))?;
            self.stdout.queue(style::Print(formatted_line))?;
        }
        Ok(())
    }

    fn draw_status_line(&mut self) -> Result<()> {
        let mode = format!(" {:?} ", self.mode).to_uppercase();
        let file = format!(" {} ", self.buffer.file.as_deref().unwrap_or("new file"));
        let pos = format!(" {}:{} ", self.cursor.row + 1, self.cursor.col + 1);
        let file_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - 2;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(Color::Black).bold().on(Color::Blue),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            "".with(Color::Blue).on(Color::Black),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize)
                .with(Color::Black)
                .bold()
                .on(Color::Black),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            "".with(Color::Blue).on(Color::Black),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            pos.with(Color::Black).bold().on(Color::Blue),
        ))?;
        Ok(())
    }

    fn handle_event(&mut self, event: event::Event) -> Result<Option<Action>> {
        if let event::Event::Resize(width, height) = event {
            self.size = (width, height);
            return Ok(None);
        }
        match self.mode {
            Mode::Normal => self.handle_normal_event(event),
            Mode::Insert => self.handle_insert_event(event),
        }
    }

    fn handle_normal_event(&self, event: event::Event) -> Result<Option<Action>> {
        log!("Event: {:?}", event);
        match event {
            event::Event::Key(key) => {
                let modifier = key.modifiers;
                match key.code {
                    KeyCode::Char('q') => Ok(Some(Action::Quit)),
                    KeyCode::Up | KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                    KeyCode::Down | KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                    KeyCode::Left | KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                    KeyCode::Right | KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
                    KeyCode::Home | KeyCode::Char('0') => Ok(Some(Action::MoveToLineStart)),
                    KeyCode::End | KeyCode::Char('$') => Ok(Some(Action::MoveToLineEnd)),
                    KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                    KeyCode::Char('b') if modifier == KeyModifiers::CONTROL => {
                        Ok(Some(Action::PageUp))
                    }
                    KeyCode::Char('f') if modifier == KeyModifiers::CONTROL => {
                        Ok(Some(Action::PageDown))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn handle_insert_event(&self, event: event::Event) -> Result<Option<Action>> {
        match event {
            event::Event::Key(key) => match key.code {
                KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                KeyCode::Char(c) => Ok(Some(Action::InsertChar(c))),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}
