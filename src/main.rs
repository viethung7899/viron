use anyhow::{Ok, Result};
use std::io::{Stdout, Write};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, read},
    style, terminal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Pointer {
    row: u16,
    col: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
}

fn handle_event(mode: &Mode, stdout: &mut Stdout, event: event::Event) -> Result<Option<Action>> {
    match mode {
        Mode::Normal => handle_normal_event(event),
        Mode::Insert => handle_insert_event(event, stdout),
    }
}

fn handle_normal_event(event: event::Event) -> Result<Option<Action>> {
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

fn handle_insert_event(event: event::Event, stdout: &mut Stdout) -> Result<Option<Action>> {
    match event {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
            event::KeyCode::Char(c) => {
                stdout.queue(style::Print(c))?;
                Ok(None)
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    let mut pointer = Pointer::default();
    let mut mode = Mode::Normal;

    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    loop {
        stdout.queue(cursor::MoveTo(pointer.col, pointer.row))?;
        stdout.flush()?;

        if let Some(action) = handle_event(&mode, &mut stdout, read()?)? {
            match action {
                Action::Quit => break,
                Action::MoveUp => {
                    pointer.row = pointer.row.saturating_sub(1);
                }
                Action::MoveLeft => {
                    pointer.col = pointer.col.saturating_sub(1);
                }
                Action::MoveRight => {
                    pointer.col += 1;
                }
                Action::MoveDown => {
                    pointer.row += 1;
                }
                Action::EnterMode(new_mode) => {
                    mode = new_mode;
                }
                _ => {}
            }
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
