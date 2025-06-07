use std::{
    io::{Stdout, Write, stdout},
    ops::Range,
};

use crate::{
    buffer::Buffer,
    log,
    theme::{Style, Theme},
};
use anyhow::Result;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, KeyCode, KeyModifiers},
    style::{self, Color, StyledContent, Stylize},
    terminal,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn get_status_color(&self) -> style::Color {
        match self {
            Mode::Normal => Color::DarkBlue,
            Mode::Insert => Color::DarkGreen,
        }
    }
}

#[derive(Debug)]
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
    SetWaitingCmd(char),
    CancelWaitingCmd,

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
    theme: Theme,
    gutter_width: u16,
    buffer: Buffer,
    stdout: Stdout,
    offset: Offset,
    cursor: Position,
    mode: Mode,
    size: (u16, u16),
    waiting_cmd: Option<char>,
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
    pub fn new(theme: Theme, buffer: Buffer) -> Result<Self> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let gutter_width = buffer.len().to_string().len() as u16 + 2;

        Ok(Self {
            theme,
            stdout,
            buffer,
            gutter_width,
            offset: Offset::default(),
            cursor: Position::default(),
            mode: Mode::Normal,
            size: terminal::size()?,
            waiting_cmd: None,
            undo: UndoStack::default(),
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
                    self.execute(&action);
                }
            }
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

    pub fn get_current_line_index(&self) -> usize {
        self.offset.top + self.cursor.row as usize
    }

    fn draw(&mut self) -> Result<()> {
        self.hide_cursor()?;
        self.draw_gutter()?;
        self.draw_viewport()?;
        self.draw_status_line()?;
        self.draw_cursor()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.stdout.queue(cursor::Hide)?;
        Ok(())
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let (row, col) = self.get_current_screen_position();
        let cursor_style = match self.waiting_cmd {
            Some(_) => cursor::SetCursorStyle::SteadyUnderScore,
            None => self.mode.set_cursor_style(),
        };
        self.stdout
            .queue(cursor::MoveTo(col as u16, row as u16))?
            .queue(cursor_style)?
            .queue(cursor::Show)?;
        Ok(())
    }

    fn draw_gutter(&mut self) -> Result<()> {
        let (_, height) = self.get_viewport_size();
        let style = self.theme.editor_style.to_content_style(None);
        for i in 0..height {
            self.stdout.queue(cursor::MoveTo(0, i))?;
            let line_number = self.offset.top + i as usize + 1;
            let content = if line_number <= self.buffer.len() {
                let w = (self.gutter_width - 1) as usize;
                format!("{line_number:>w$} ")
            } else {
                " ".repeat(self.gutter_width as usize)
            };
            let styled_content = StyledContent::new(style, content);
            self.stdout
                .queue(style::PrintStyledContent(styled_content))?;
        }
        Ok(())
    }

    fn draw_viewport(&mut self) -> Result<()> {
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
                self.fill_line(row, col, &editor_style)?;
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
                self.print_char(row, col, char, &style)?;
            }
            col += 1;
        }

        while row < end_row {
            self.fill_line(row, col, &editor_style)?;
            row += 1;
            col = 0;
        }

        Ok(())
    }

    fn print_char(&mut self, row: usize, col: usize, ch: char, style: &Style) -> Result<()> {
        let row = (row - self.offset.top) as u16;
        let col = (col - self.offset.left) as u16 + self.gutter_width;
        let style = style.to_content_style(Some(&self.theme.editor_style));
        let styled_content = StyledContent::new(style, ch);
        self.stdout
            .queue(cursor::MoveTo(col, row))?
            .queue(style::PrintStyledContent(styled_content))?;
        Ok(())
    }

    fn fill_line(&mut self, row: usize, col: usize, style: &Style) -> Result<()> {
        let col = col.max(self.offset.left);
        let row = (row - self.offset.top) as u16;
        let col = (col - self.offset.left) as u16;
        let (viewport_width, _) = self.get_viewport_size();
        let repeat = (viewport_width.saturating_sub(col)) as usize;
        let style = style.to_content_style(Some(&self.theme.editor_style));
        let styled_content = StyledContent::new(style, " ".repeat(repeat));
        self.stdout
            .queue(cursor::MoveTo(col + self.gutter_width, row))?
            .queue(style::PrintStyledContent(styled_content))?;
        Ok(())
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
                let content = &code[range.clone()];

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

    fn draw_status_line(&mut self) -> Result<()> {
        let mode = format!(" {:?} ", self.mode).to_uppercase();
        let file = format!(" {} ", self.buffer.file.as_deref().unwrap_or("new file"));
        let pos = format!(" {}:{} ", self.cursor.row + 1, self.cursor.col + 1);
        let file_width = self.size.0 - mode.len() as u16 - pos.len() as u16;

        let color = self.mode.get_status_color();
        let black = Color::Black;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(color).bold().negative(),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize)
                .bold()
                .on(black),
        ))?;
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
                    KeyCode::Char('u') => Some(Action::Undo),
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
                    KeyCode::Left | KeyCode::Char('h') => Some(Action::MoveLeft),
                    KeyCode::Right | KeyCode::Char('l') => Some(Action::MoveRight),
                    KeyCode::Home | KeyCode::Char('0') => Some(Action::MoveToLineStart),
                    KeyCode::End | KeyCode::Char('$') => Some(Action::MoveToLineEnd),
                    KeyCode::Char('G') => Some(Action::MoveToBottom),
                    KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
                    KeyCode::Char('o') => Some(Action::InsertLineBelowCursor),
                    KeyCode::Char('O') => Some(Action::InsertLineAtCursor),
                    KeyCode::Char('x') => Some(Action::DeleteCharAtCursor),
                    KeyCode::Char('b') if modifier == KeyModifiers::CONTROL => Some(Action::PageUp),
                    KeyCode::Char('f') if modifier == KeyModifiers::CONTROL => {
                        Some(Action::PageDown)
                    }
                    KeyCode::Char(c) => Some(Action::SetWaitingCmd(c)),
                    KeyCode::Esc => Some(Action::CancelWaitingCmd),
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
            'z' => match event {
                event::Event::Key(key) => match key.code {
                    KeyCode::Char('z') => Some(Action::MoveToViewportCenter),
                    _ => None,
                },
                _ => None,
            },
            'g' => match event {
                event::Event::Key(key) => match key.code {
                    KeyCode::Char('g') => Some(Action::MoveToTop),
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
                KeyCode::Up => Some(Action::MoveUp),
                KeyCode::Down => Some(Action::MoveDown),
                KeyCode::Left => Some(Action::MoveLeft),
                KeyCode::Right => Some(Action::MoveRight),
                _ => None,
            },
            _ => None,
        }
    }

    fn execute(&mut self, action: &Action) {
        match action {
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
                let line = self.get_current_line_index();
                let offset = self.cursor.col as usize;
                self.undo.record(Action::DeleteChatAt(line, offset));
                self.buffer.insert(line, offset, *char);
                self.cursor.col += 1;
            }
            Action::InsertLineAt(line, content) => {
                self.buffer.insert_line(*line, content.to_string());
            }
            Action::InsertLineAtCursor => {
                self.undo
                    .push(Action::DeleteLineAt(self.get_current_line_index()));
                self.execute(&Action::InsertLineAt(
                    self.get_current_line_index(),
                    "".into(),
                ));
                self.mode = Mode::Insert;
            }
            Action::InsertLineBelowCursor => {
                self.undo
                    .push(Action::DeleteLineAt(self.get_current_line_index() + 1));
                self.execute(&Action::InsertLineAt(
                    self.get_current_line_index() + 1,
                    "".into(),
                ));
                self.cursor.row += 1;
                self.mode = Mode::Insert;
            }
            Action::DeleteCharAtCursor => {
                let line = self.get_current_line_index();
                let offset = self.cursor.col;
                self.buffer.remove(line as usize, offset as usize);
            }
            Action::DeleteChatAt(line, offset) => {
                self.buffer.remove(*line, *offset);
            }
            Action::DeleteCurrentLine => {
                let line_index = self.get_current_line_index() as usize;
                if let Some(content) = self.buffer.get_line(line_index) {
                    self.buffer.remove_line(line_index);
                    self.undo.push(Action::InsertLineAt(line_index, content));
                }
            }
            Action::DeleteLineAt(line) => {
                self.buffer.remove_line(*line);
            }
            Action::SetWaitingCmd(char) => {
                self.waiting_cmd = Some(*char);
            }
            Action::CancelWaitingCmd => {
                self.waiting_cmd = None;
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
            _ => {}
        }
    }
}
