use std::io;
use anyhow::Result;
use crossterm::{ExecutableCommand, cursor, style, terminal};
use std::io::Stdout;

pub struct TerminalContext {
    pub width: usize,
    pub height: usize,
    pub stdout: Stdout,
}

impl TerminalContext {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(cursor::Hide)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let (width, height) = terminal::size()?;

        Ok(Self {
            width: width as usize,
            height: height as usize,
            stdout,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.width = width;
        self.height = height;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn cleanup(mut self) -> Result<()> {
        self.stdout
            .execute(style::ResetColor)?
            .execute(cursor::Show)?
            .execute(cursor::SetCursorStyle::DefaultUserShape)?
            .execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
