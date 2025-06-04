use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::{ExecutableCommand, QueueableCommand, cursor, event, style, terminal};

#[derive(Debug, Default)]
pub struct Editor {
    cursor: Cursor,
    mode: Mode,
}

#[derive(Debug, Clone, Default)]
struct Cursor {
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

impl Editor {
    pub fn draw(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        stdout.queue(cursor::MoveTo(self.cursor.col, self.cursor.row))?;
        stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;

        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        loop {
            self.draw(&mut stdout)?;
            if let Some(action) = self.handle_event(event::read()?)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.cursor.row = self.cursor.row.saturating_sub(1);
                    }
                    Action::MoveLeft => {
                        self.cursor.col = self.cursor.col.saturating_sub(1);
                    }
                    Action::MoveRight => {
                        self.cursor.col += 1;
                    }
                    Action::MoveDown => {
                        self.cursor.row += 1;
                    }
                    Action::EnterMode(mode) => {
                        self.mode = mode;
                    }
                    Action::InsertChar(c) => {
                        stdout.queue(cursor::MoveTo(self.cursor.col, self.cursor.row))?;
                        stdout.queue(style::Print(c))?;
                        self.cursor.col += 1;
                    }
                }
            }
        }

        stdout.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn handle_event(&self, event: event::Event) -> Result<Option<Action>> {
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
