use std::io::{Stdout, Write, stdout};

use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor, event,
    style::{self, Color, Stylize},
    terminal,
};

use crate::buffer::Buffer;

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
            self.draw()?;
            if let Some(action) = self.handle_event(event::read()?)? {
                let (viewport_width, viewport_height) = self.get_viewport_size();
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.cursor.row = self.cursor.row.saturating_sub(1);
                    }
                    Action::MoveLeft => {
                        self.cursor.col = self.cursor.col.saturating_sub(1);
                    }
                    Action::MoveRight => {
                        let line_length = self
                            .get_viewport_line(self.cursor.row)
                            .map_or(0, |line| line.len() as u16);
                        if self.cursor.col + 1 < viewport_width && self.cursor.col + 1 < line_length
                        {
                            self.cursor.col += 1;
                        }
                    }
                    Action::MoveDown => {
                        if self.cursor.row + 1 < viewport_height {
                            self.cursor.row += 1;
                        }
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

    fn get_viewport_line(&self, line: u16) -> Option<String> {
        let buffer_line = (self.offset.row + line) as usize;
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
        match event {
            event::Event::Key(key) => match key.code {
                event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                event::KeyCode::Up | event::KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                event::KeyCode::Down | event::KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                event::KeyCode::Left | event::KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Right | event::KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
                event::KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn handle_insert_event(&self, event: event::Event) -> Result<Option<Action>> {
        match event {
            event::Event::Key(key) => match key.code {
                event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                event::KeyCode::Char(c) => Ok(Some(Action::InsertChar(c))),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}
