mod render_buffer;

use std::{
    io::{Stdout, Write, stdout},
    ops::Range,
    time::Instant,
};

use crate::{
    buffer::Buffer,
    config::{Config, KeyAction, KeyMapping, get_key_action},
    editor::render_buffer::{Change, RenderBuffer},
    highlighter::Highlighter,
    log,
    theme::{Style, Theme},
};
use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode},
    style::{self, StyledContent},
    terminal::{self},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct StyleInfo {
    pub range: Range<usize>,
    pub style: Style,
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
struct Offset {
    pub top: usize,
    pub left: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Quit,
    Undo,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    PageUp,
    PageDown,

    MoveToTop,
    MoveToBottom,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToViewportCenter,

    EnterMode(Mode),

    InsertCharAtCursor(char),
    InsertLineAt(usize, String),
    InsertLineBelowCursor,
    InsertLineAtCursor,
    DeleteCharAtCursor,
    DeleteChatAt(usize, usize),
    DeleteCurrentLine,
    DeleteLineAt(usize),

    Multiple(Vec<Action>),
}

#[derive(Debug, Default)]
struct UndoStack {
    actions: Vec<Action>,
    buffer: Vec<Action>,
}

impl UndoStack {
    fn record(&mut self, undo_action: Action) {
        self.buffer.push(undo_action);
    }

    fn push(&mut self, undo_action: Action) {
        self.batch();
        self.actions.push(undo_action);
    }

    fn batch(&mut self) {
        let mut reversed = Vec::new();
        while let Some(action) = self.buffer.pop() {
            reversed.push(action);
        }
        if !reversed.is_empty() {
            self.actions.push(Action::Multiple(reversed))
        }
    }

    fn pop(&mut self) -> Option<Action> {
        self.batch();
        self.actions.pop()
    }
}

pub struct Editor {
    config: Config,
    theme: Theme,
    highlighter: Highlighter,
    gutter_width: u16,
    buffer: Buffer,
    stdout: Stdout,
    offset: Offset,
    cursor: Position,
    mode: Mode,
    size: (u16, u16),
    waiting_key_action: Option<KeyMapping>,
    undo: UndoStack,
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.flush();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    pub fn new_with_size(
        width: u16,
        height: u16,
        config: Config,
        theme: Theme,
        buffer: Buffer,
    ) -> Result<Self> {
        let stdout = stdout();
        terminal::enable_raw_mode().unwrap();
        let gutter_width = buffer.len().to_string().len() as u16 + 2;
        let highlighter = Highlighter::new()?;

        Ok(Self {
            config,
            theme,
            highlighter,
            stdout,
            buffer,
            gutter_width,
            offset: Offset::default(),
            cursor: Position::default(),
            mode: Mode::Normal,
            size: (width, height),
            waiting_key_action: None,
            undo: UndoStack::default(),
        })
    }

    pub fn new(config: Config, theme: Theme, current_buffer: Buffer) -> Result<Self> {
        let (width, height) = terminal::size()?;
        Self::new_with_size(width, height, config, theme, current_buffer)
    }

    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let (width, height) = self.size;
        let mut buffer = RenderBuffer::new(
            width as usize,
            height as usize,
            Some(self.theme.editor_style.clone()),
        );

        self.render(&mut buffer)?;

        loop {
            let current_buffer = buffer.clone();
            let event = event::read()?;
            let start = Instant::now();

            let action_start = Instant::now();
            if let Some(key_action) = self.handle_event(event) {
                log!("Key action = {key_action:?}");
                let quit = match key_action {
                    KeyAction::Single(action) => self.execute(&action),
                    KeyAction::Multiple(actions) => {
                        let mut quit = false;
                        for action in actions {
                            quit |= self.execute(&action);
                            if quit {
                                break;
                            }
                        }
                        quit
                    }
                    KeyAction::Nested(mapping) => {
                        self.waiting_key_action = Some(mapping);
                        false
                    }
                };
                if quit {
                    break;
                }
            }
            let changed = self.reset_bounds();
            if changed {
                self.draw_viewport(&mut buffer)?;
                self.draw_gutter(&mut buffer);
            }
            log!("Action takes {:?}", action_start.elapsed());

            self.stdout.execute(cursor::Hide)?;
            self.draw_status_line(&mut buffer);
            self.render_diff(buffer.diff(&current_buffer))?;
            self.draw_cursor()?;
            self.stdout.execute(cursor::Show)?;
            log!("Action takes {:?}", start.elapsed());
        }

        Ok(())
    }

    fn get_viewport_size(&self) -> (u16, u16) {
        let (width, height) = self.size;
        (width - self.gutter_width, height - 2)
    }

    fn get_current_screen_position(&self) -> (u16, u16) {
        let row = self.cursor.row - self.offset.top;
        let col = self.cursor.col - self.offset.left;
        (row as u16, col as u16 + self.gutter_width)
    }

    fn reset_bounds(&mut self) -> bool {
        // Reset the column
        let mut changed = false;
        let current_line = self.buffer.get_line(self.cursor.row);
        let mut current_length = current_line.clone().map_or(0, |line| line.len());
        if self.mode != Mode::Insert {
            current_length = current_length.saturating_sub(1);
        }
        self.cursor.col = self.cursor.col.min(current_length);

        // Reset the offset
        let (width, height) = self.get_viewport_size();

        if self.cursor.row < self.offset.top {
            self.offset.top = self.cursor.row;
            changed = true;
        }

        if self.cursor.row >= self.offset.top + height as usize {
            self.offset.top = self
                .cursor
                .row
                .saturating_sub(height.saturating_sub(1) as usize);
            changed = true;
        }

        if self.cursor.col < self.offset.left {
            self.offset.left = self.cursor.col;
            changed = true;
        }

        if self.cursor.col >= self.offset.left + width as usize {
            self.offset.left = self
                .cursor
                .col
                .saturating_sub(width.saturating_sub(1) as usize);
            changed = true;
        }
        changed
    }

    fn render(&mut self, buffer: &mut RenderBuffer) -> Result<()> {
        self.draw_viewport(buffer)?;
        self.draw_gutter(buffer);
        self.draw_status_line(buffer);

        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?;

        for cell in buffer.cells.iter() {
            let style = cell.style.to_content_style(Some(&self.theme.editor_style));
            let content = StyledContent::new(style, cell.c);
            self.stdout.queue(style::Print(content))?;
        }

        self.draw_cursor()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn render_diff(&mut self, changes: Vec<Change>) -> Result<()> {
        let start = Instant::now();
        for change in changes {
            let style = change
                .cell
                .style
                .to_content_style(Some(&self.theme.editor_style));
            let content = StyledContent::new(style, change.cell.c);
            self.stdout
                .queue(cursor::MoveTo(change.x as u16, change.y as u16))?
                .queue(style::Print(content))?;
        }
        self.stdout.flush()?;

        log!("Render diff took: {:?}", start.elapsed());
        Ok(())

        // Implement rendering logic here
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let (row, col) = self.get_current_screen_position();
        let cursor_style = match self.waiting_key_action {
            Some(_) => cursor::SetCursorStyle::SteadyUnderScore,
            None => self.mode.set_cursor_style(),
        };
        self.stdout
            .queue(cursor_style)?
            .queue(cursor::MoveTo(col as u16, row as u16))?;
        Ok(())
    }

    fn draw_gutter(&mut self, buffer: &mut RenderBuffer) {
        let (_, height) = self.get_viewport_size();
        for i in 0..height {
            let line_number = self.offset.top + i as usize + 1;
            let content = if line_number <= self.buffer.len() {
                let w = (self.gutter_width - 1) as usize;
                format!("{line_number:>w$} ")
            } else {
                " ".repeat(self.gutter_width as usize)
            };
            buffer.set_text(0, i as usize, &content, &self.theme.gutter_style);
        }
    }

    fn draw_viewport(&mut self, buffer: &mut RenderBuffer) -> Result<()> {
        let (width, height) = self.get_viewport_size();
        let viewport_buffer = self
            .buffer
            .get_viewport_buffer(self.offset.top, height as usize);
        let styles = self.highlight(&viewport_buffer)?;
        let editor_style = self.theme.editor_style.clone();

        let mut row = self.offset.top;
        let end_row = self.offset.top + height as usize;
        let mut col = 0 as usize;

        for (index, char) in viewport_buffer.chars().enumerate() {
            if char == '\n' {
                self.fill_line(buffer, row, col);
                row += 1;
                col = 0;

                if row > end_row {
                    break;
                }
                continue;
            }

            if col >= self.offset.left && col < self.offset.left + width as usize {
                let style = styles
                    .iter()
                    .find(|c| c.range.contains(&index))
                    .map(|c| c.style.clone())
                    .unwrap_or(editor_style.clone());
                self.print_char(buffer, row, col, char, &style);
            }
            col += 1;
        }

        while row < end_row {
            self.fill_line(buffer, row, col);
            row += 1;
            col = 0;
        }

        Ok(())
    }

    fn print_char(
        &mut self,
        buffer: &mut RenderBuffer,
        row: usize,
        col: usize,
        c: char,
        style: &Style,
    ) {
        let y = row - self.offset.top;
        let x = col - self.offset.left + self.gutter_width as usize;
        buffer.set_cell(x, y, c, style);
    }

    fn fill_line(&mut self, buffer: &mut RenderBuffer, row: usize, col: usize) {
        let row = row - self.offset.top;
        let col = col.saturating_sub(self.offset.left);
        let width = self.get_viewport_size().0 as usize;
        let content = " ".repeat(width.saturating_sub(col));
        let y = row;
        let x = col + self.gutter_width as usize;
        buffer.set_text(x, y, &content, &self.theme.editor_style);
    }

    fn highlight(&mut self, code: &str) -> Result<Vec<StyleInfo>> {
        self.highlighter.highlight(code, &self.theme)
    }

    fn draw_status_line(&mut self, buffer: &mut RenderBuffer) {
        let left = format!(" {:?} ", self.mode).to_uppercase();
        let right = format!(" {}:{} ", self.cursor.row + 1, self.cursor.col + 1);
        let file = format!(" {}", self.buffer.file.as_deref().unwrap_or("new file"));
        let center_width = self.size.0 as usize - left.len() - right.len();
        let center = format!("{file:<center_width$}");

        let outer_style = match self.mode {
            Mode::Insert => &self.theme.status_line_style.insert,
            Mode::Normal => &self.theme.status_line_style.normal,
        };

        let y = self.size.1 as usize - 2;
        buffer.set_text(0, y, &left, &outer_style);
        buffer.set_text(left.len(), y, &center, &self.theme.status_line_style.inner);
        buffer.set_text(left.len() + center_width, y, &right, &outer_style);
    }

    fn handle_event(&mut self, event: Event) -> Option<KeyAction> {
        if let Event::Resize(width, height) = event {
            self.size = (width, height);
            return None;
        }

        if let Some(mapping) = &self.waiting_key_action {
            let action = get_key_action(mapping, &event);
            self.waiting_key_action = None;
            return action;
        }

        match self.mode {
            Mode::Insert => self.handle_insert_event(&event),
            Mode::Normal => self.handle_normal_event(&event),
        }
    }

    fn handle_normal_event(&mut self, event: &Event) -> Option<KeyAction> {
        get_key_action(&self.config.keys.normal, &event)
    }

    fn handle_insert_event(&self, event: &Event) -> Option<KeyAction> {
        let action = get_key_action(&self.config.keys.insert, &event);

        if action.is_some() {
            return action;
        }

        match event {
            Event::Key(event) => match event.code {
                KeyCode::Char(c) => KeyAction::Single(Action::InsertCharAtCursor(c)).into(),
                _ => None,
            },
            _ => None,
        }
    }

    fn execute(&mut self, action: &Action) -> bool {
        match action {
            Action::Quit => {
                return true;
            }
            Action::MoveUp => {
                self.cursor.row = self.cursor.row.saturating_sub(1);
            }
            Action::MoveDown => {
                self.cursor.row = self.buffer.len().saturating_sub(1).min(self.cursor.row + 1);
            }
            Action::MoveLeft => {
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            Action::MoveRight => {
                self.cursor.col += 1;
            }
            Action::PageUp => {
                let (_, height) = self.get_viewport_size();
                self.cursor.row = self.cursor.row.saturating_sub(height as usize);
            }
            Action::PageDown => {
                let (_, height) = self.get_viewport_size();
                self.cursor.row = self
                    .buffer
                    .len()
                    .saturating_sub(1)
                    .min(self.cursor.row + height as usize);
            }
            Action::MoveToTop => {
                self.cursor.row = 0;
            }
            Action::MoveToBottom => {
                self.cursor.row = self.buffer.len().saturating_sub(1);
            }
            Action::MoveToLineStart => {
                self.cursor.col = 0;
            }
            Action::MoveToLineEnd => {
                self.cursor.col = self.buffer.get_line(self.cursor.row).map_or(0, |s| s.len())
            }
            Action::MoveToViewportCenter => {
                let (_, height) = self.get_viewport_size();
                self.offset.top = self.cursor.row.saturating_sub(height as usize / 2);
            }
            Action::EnterMode(mode) => {
                match (self.mode, mode) {
                    (Mode::Insert, Mode::Normal) => {
                        self.undo.batch();
                    }
                    _ => {}
                }
                self.mode = *mode;
            }
            Action::InsertCharAtCursor(char) => {
                let line = self.cursor.row;
                let offset = self.cursor.col;
                self.undo.record(Action::DeleteChatAt(line, offset));
                self.buffer.insert(line, offset, *char);
                self.cursor.col += 1;
            }
            Action::InsertLineAt(line, content) => {
                self.buffer.insert_line(*line, content.to_string());
            }
            Action::InsertLineAtCursor => {
                let line = self.cursor.row;
                self.undo.push(Action::DeleteLineAt(line));
                self.execute(&Action::InsertLineAt(line, "".into()));
            }
            Action::InsertLineBelowCursor => {
                let line = self.cursor.row;
                self.undo.push(Action::DeleteLineAt(line + 1));
                self.execute(&Action::InsertLineAt(line + 1, "".into()));
                self.cursor.row += 1;
            }
            Action::DeleteCharAtCursor => {
                let line = self.cursor.row;
                let offset = self.cursor.col;
                self.buffer.remove(line as usize, offset as usize);
            }
            Action::DeleteChatAt(line, offset) => {
                self.buffer.remove(*line, *offset);
            }
            Action::DeleteCurrentLine => {
                let line = self.cursor.row;
                if let Some(content) = self.buffer.get_line(line) {
                    self.buffer.remove_line(line);
                    self.undo.push(Action::InsertLineAt(line, content));
                }
            }
            Action::DeleteLineAt(line) => {
                self.buffer.remove_line(*line);
            }
            Action::Undo => {
                if let Some(action) = self.undo.pop() {
                    self.execute(&action);
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    if self.execute(&action) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
