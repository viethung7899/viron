use std::io::{Stdout, Write, stdout};

use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, KeyCode, KeyModifiers},
    style::{self, Color, Stylize},
    terminal,
};

use crate::action::Action;
use crate::{buffer::Buffer, log};

#[derive(Debug)]
pub struct Editor {
    pub(crate) stdout: Stdout,
    pub(crate) buffer: Buffer,
    pub(crate) offset: Position,
    pub(crate) cursor: Position,
    pub(crate) mode: Mode,
    pub(crate) size: (u16, u16),
    pub(crate) waiting_cmd: Option<char>,
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

impl Mode {
    fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Normal => cursor::SetCursorStyle::SteadyBlock,
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
        }
    }

    fn get_status_color(&self) -> style::Color {
        match self {
            Mode::Normal => Color::DarkBlue,
            Mode::Insert => Color::DarkGreen,
        }
    }
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
            waiting_cmd: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.reset_bounds();
            self.draw()?;
            if let Some(action) = self.handle_event(event::read()?) {
                if matches!(action, Action::Quit) {
                    break;
                } else {
                    action.execute(self);
                }
            }
        }

        Ok(())
    }

    pub fn get_viewport_size(&self) -> (u16, u16) {
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

    pub fn get_buffer_line_index(&self) -> u16 {
        self.offset.row + self.cursor.row
    }

    pub fn get_viewport_line(&self, viewport_line: u16) -> Option<String> {
        let buffer_line = self.offset.row + viewport_line;
        self.buffer.get_line(buffer_line as usize)
    }

    fn draw(&mut self) -> Result<()> {
        self.set_cursor_style()?;
        self.draw_viewport()?;
        self.draw_status_line()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.col, self.cursor.row))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn set_cursor_style(&mut self) -> Result<()> {
        let cursor = match self.waiting_cmd {
            Some(_) => cursor::SetCursorStyle::SteadyUnderScore,
            None => self.mode.set_cursor_style(),
        };
        self.stdout.queue(cursor)?;
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

        let color = self.mode.get_status_color();
        let black = Color::Black;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(color).bold().negative(),
        ))?;
        self.stdout
            .queue(style::PrintStyledContent("".with(color).on(black)))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize)
                .bold()
                .on(black),
        ))?;
        self.stdout
            .queue(style::PrintStyledContent("".with(color).on(black)))?;
        self.stdout
            .queue(style::PrintStyledContent(pos.with(color).bold().negative()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: event::Event) -> Option<Action> {
        if let event::Event::Resize(width, height) = event {
            self.size = (width, height);
            return None;
        }
        match self.mode {
            Mode::Normal => self.handle_normal_event(&event),
            Mode::Insert => self.handle_insert_event(&event),
        }
    }

    fn handle_normal_event(&mut self, event: &event::Event) -> Option<Action> {
        log!("Event: {:?}", event);

        let action = self
            .waiting_cmd
            .and_then(|cmd| self.handle_waiting_command(cmd, &event));

        if action.is_some() {
            self.waiting_cmd = None;
            return action;
        }

        match event {
            event::Event::Key(key) => {
                let modifier = key.modifiers;
                match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
                    KeyCode::Left | KeyCode::Char('h') => Some(Action::MoveLeft),
                    KeyCode::Right | KeyCode::Char('l') => Some(Action::MoveRight),
                    KeyCode::Home | KeyCode::Char('0') => Some(Action::MoveToLineStart),
                    KeyCode::End | KeyCode::Char('$') => Some(Action::MoveToLineEnd),
                    KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
                    KeyCode::Char('x') => Some(Action::DeleteCharAtCursor),
                    KeyCode::Char('b') if modifier == KeyModifiers::CONTROL => Some(Action::PageUp),
                    KeyCode::Char('f') if modifier == KeyModifiers::CONTROL => {
                        Some(Action::PageDown)
                    }
                    KeyCode::Char(c) => Some(Action::SetWaitingCmd(c)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn handle_waiting_command(&self, command: char, event: &event::Event) -> Option<Action> {
        match command {
            'd' => match event {
                event::Event::Key(key) => match key.code {
                    KeyCode::Char('d') => Some(Action::DeleteCurrentLine),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn handle_insert_event(&self, event: &event::Event) -> Option<Action> {
        match event {
            event::Event::Key(key) => match key.code {
                KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
                KeyCode::Char(c) => Some(Action::InsertCharAtCursor(c)),
                _ => None,
            },
            _ => None,
        }
    }
}
