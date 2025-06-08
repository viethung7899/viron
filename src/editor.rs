mod render_buffer;

use std::{
    io::{Stdout, Write, stdout},
    ops::Range,
};

use crate::{
    buffer::Buffer,
    config::{Config, KeyAction, KeyMapping, get_key_action},
    editor::render_buffer::RenderBuffer,
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
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
use tree_sitter_rust::HIGHLIGHTS_QUERY;

#[derive(Debug, Clone)]
pub struct StyleInfo {
    range: Range<usize>,
    style: Style,
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

#[derive(Debug)]
pub struct Editor {
    config: Config,
    theme: Theme,
    gutter_width: u16,
    buffer: RenderBuffer,
    last_buffer: Option<RenderBuffer>,
    current_buffer: Buffer,
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
    pub fn new(config: Config, theme: Theme, current_buffer: Buffer) -> Result<Self> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let size = terminal::size()?;
        let gutter_width = current_buffer.len().to_string().len() as u16 + 2;
        let style = theme.editor_style.clone();
        let buffer = RenderBuffer::new(size.0 as usize, size.1 as usize, Some(style));

        Ok(Self {
            config,
            theme,
            stdout,
            current_buffer,
            buffer,
            last_buffer: None,
            gutter_width,
            offset: Offset::default(),
            cursor: Position::default(),
            mode: Mode::Normal,
            size,
            waiting_key_action: None,
            undo: UndoStack::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.render()?;
        loop {
            self.reset_bounds();
            // self.draw()?;
            if let Some(key_action) = self.handle_event(event::read()?) {
                log!("Key action = {:?}", key_action);
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

            // self.stdout.execute(cursor::Hide)?;
            self.render_diff()?;
            self.draw_cursor()?;
            self.stdout.execute(cursor::Show)?;
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

    fn reset_bounds(&mut self) {
        // Reset the column
        let current_line = self.current_buffer.get_line(self.cursor.row);
        let mut current_length = current_line.clone().map_or(0, |line| line.len());
        if self.mode != Mode::Insert {
            current_length = current_length.saturating_sub(1);
        }
        self.cursor.col = self.cursor.col.min(current_length);

        // Reset the offset
        let (width, height) = self.get_viewport_size();

        if self.cursor.row < self.offset.top {
            self.offset.top = self.cursor.row;
        }

        if self.cursor.row >= self.offset.top + height as usize {
            self.offset.top = self
                .cursor
                .row
                .saturating_sub(height.saturating_sub(1) as usize);
        }

        if self.cursor.col < self.offset.left {
            self.offset.left = self.cursor.col;
        }

        if self.cursor.col >= self.offset.left + width as usize {
            self.offset.left = self
                .cursor
                .col
                .saturating_sub(width.saturating_sub(1) as usize);
        }
    }

    fn render(&mut self) -> Result<()> {
        self.draw_viewport()?;
        self.draw_gutter();
        self.draw_status_line();

        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?;

        for cell in self.buffer.cells.iter() {
            let style = cell.style.to_content_style(Some(&self.theme.editor_style));
            let content = StyledContent::new(style, cell.c);
            self.stdout.queue(style::Print(content))?;
        }

        self.draw_cursor()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn render_diff(&mut self) -> Result<()> {
        let Some(ref last_buffer) = self.last_buffer else {
            return self.render();
        };

        let changes = self.buffer.diff(&last_buffer);
        for change in changes {
            let x = change.x + self.gutter_width as usize;
            let y = change.y + self.offset.top;
            let style = change
                .cell
                .style
                .to_content_style(Some(&self.theme.editor_style));
            let content = StyledContent::new(style, change.cell.c);
            self.stdout
                .queue(cursor::MoveTo(x as u16, y as u16))?
                .queue(style::Print(content))?;
        }
        self.stdout.flush()?;
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

    fn draw_gutter(&mut self) {
        let (_, height) = self.get_viewport_size();
        for i in 0..height {
            let line_number = self.offset.top + i as usize + 1;
            let content = if line_number <= self.current_buffer.len() {
                let w = (self.gutter_width - 1) as usize;
                format!("{line_number:>w$} ")
            } else {
                " ".repeat(self.gutter_width as usize)
            };
            self.buffer
                .set_text(0, i as usize, &content, &self.theme.gutter_style);
        }
    }

    fn draw_viewport(&mut self) -> Result<()> {
        let (width, height) = self.get_viewport_size();
        let viewport_buffer = self
            .current_buffer
            .get_viewport_buffer(self.offset.top, height as usize);
        let styles = self.highlight(&viewport_buffer)?;
        let editor_style = self.theme.editor_style.clone();

        let mut row = self.offset.top;
        let end_row = self.offset.top + height as usize;
        let mut col = 0 as usize;

        for (index, char) in viewport_buffer.chars().enumerate() {
            if char == '\n' {
                self.fill_line(row, col);
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
                self.print_char(row, col, char, &style);
            }
            col += 1;
        }

        while row < end_row {
            self.fill_line(row, col);
            row += 1;
            col = 0;
        }

        Ok(())
    }

    fn print_char(&mut self, row: usize, col: usize, c: char, style: &Style) {
        let y = row - self.offset.top;
        let x = col - self.offset.left + self.gutter_width as usize;
        self.buffer.set_cell(x, y, c, style);
    }

    fn fill_line(&mut self, row: usize, col: usize) {
        let row = row - self.offset.top;
        let col = col.saturating_sub(self.offset.left);
        let width = self.get_viewport_size().0 as usize;
        let content = " ".repeat(width.saturating_sub(col));
        let y = row;
        let x = col + self.gutter_width as usize;
        self.buffer
            .set_text(x, y, &content, &self.theme.editor_style);
    }

    fn highlight(&self, code: &str) -> Result<Vec<StyleInfo>> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE.into();
        parser.set_language(&language)?;
        let tree = parser.parse(code, None).expect("Grammar tree exists");
        let query = Query::new(&language, HIGHLIGHTS_QUERY).expect("Rust highlight query exists");

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), code.as_bytes());
        let mut colors = Vec::new();

        while let Some(matching) = matches.next() {
            for capture in matching.captures {
                let node = capture.node;
                let range = node.byte_range();

                let scope = query.capture_names()[capture.index as usize];
                let style = self.theme.get_style(scope);
                // let content = &code[range.clone()];

                if let Some(style) = style {
                    // log!("[found] {scope} = {content}");
                    colors.push(StyleInfo {
                        range,
                        style: style,
                    });
                } else {
                    // log!("[not found] {scope} = {content}");
                }
            }
        }
        Ok(colors)
    }

    fn draw_status_line(&mut self) {
        let left = format!(" {:?} ", self.mode).to_uppercase();
        let right = format!(" {}:{} ", self.cursor.row + 1, self.cursor.col + 1);
        let file = format!(
            " {}",
            self.current_buffer.file.as_deref().unwrap_or("new file")
        );
        let center_width = self.size.0 as usize - left.len() - right.len();
        let center = format!("{file:<center_width$}");

        let outer_style = match self.mode {
            Mode::Insert => &self.theme.status_line_style.insert,
            Mode::Normal => &self.theme.status_line_style.normal,
        };

        let y = self.size.1 as usize - 2;
        self.buffer.set_text(0, y, &left, &outer_style);
        self.buffer
            .set_text(left.len(), y, &center, &self.theme.status_line_style.inner);
        self.buffer
            .set_text(left.len() + center_width, y, &right, &outer_style);
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
                self.cursor.row = self
                    .current_buffer
                    .len()
                    .saturating_sub(1)
                    .min(self.cursor.row + 1);
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
                    .current_buffer
                    .len()
                    .saturating_sub(1)
                    .min(self.cursor.row + height as usize);
            }
            Action::MoveToTop => {
                self.cursor.row = 0;
            }
            Action::MoveToBottom => {
                self.cursor.row = self.current_buffer.len().saturating_sub(1);
            }
            Action::MoveToLineStart => {
                self.cursor.col = 0;
            }
            Action::MoveToLineEnd => {
                self.cursor.col = self
                    .current_buffer
                    .get_line(self.cursor.row)
                    .map_or(0, |s| s.len())
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
                self.current_buffer.insert(line, offset, *char);
                self.cursor.col += 1;
            }
            Action::InsertLineAt(line, content) => {
                self.current_buffer.insert_line(*line, content.to_string());
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
                self.current_buffer.remove(line as usize, offset as usize);
            }
            Action::DeleteChatAt(line, offset) => {
                self.current_buffer.remove(*line, *offset);
            }
            Action::DeleteCurrentLine => {
                let line = self.cursor.row;
                if let Some(content) = self.current_buffer.get_line(line) {
                    self.current_buffer.remove_line(line);
                    self.undo.push(Action::InsertLineAt(line, content));
                }
            }
            Action::DeleteLineAt(line) => {
                self.current_buffer.remove_line(*line);
            }
            Action::Undo => {
                if let Some(action) = self.undo.pop() {
                    self.execute(&action);
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    self.execute(&action);
                }
            }
        }
        false
    }
}
