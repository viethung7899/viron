mod command_center;
mod render_buffer;

use std::{
    io::{Stdout, Write, stdout},
    str::from_utf8,
    time::Instant,
};

use crate::{
    buffer::Buffer,
    config::{Config, KeyAction, KeyMapping, get_key_action},
    editor::{
        command_center::CommandCenter,
        render_buffer::{Change, RenderBuffer},
    },
    highlighter::Highlighter,
    log,
    theme::{Style, Theme},
};
use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent},
    style::{self, StyledContent},
    terminal::{self},
};
use serde::{Deserialize, Serialize};
use tree_sitter::Point;

#[derive(Debug, Clone, Default)]
struct Offset {
    pub top: usize,
    pub left: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Mode {
    fn to_name(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
        }
    }
    fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Normal | Mode::Command => cursor::SetCursorStyle::SteadyBlock,
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

    GotoLine(usize),

    InsertCharAtCursor(char),
    NewLineAtCursor,
    DeleteCharAtCursor,
    BackspaceCharAtCursor,
    DeleteCurrentLine,

    EnterMode(Mode),

    ExecuteCommand,

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
    file_name: Option<String>,
    highlighter: Highlighter,
    gutter_width: u16,
    buffer: Buffer,
    stdout: Stdout,
    offset: Offset,
    cursor: Point,
    mode: Mode,
    size: (u16, u16),
    waiting_key_command: Option<char>,
    waiting_key_action: Option<KeyMapping>,
    undo: UndoStack,
    command_center: CommandCenter,
    last_error: Option<String>,
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
        file_name: Option<String>,
        buffer: Buffer,
    ) -> Result<Self> {
        let stdout = stdout();
        terminal::enable_raw_mode().unwrap();
        let gutter_width = buffer.line_count().to_string().len() as u16 + 2;
        let highlighter = Highlighter::new()?;

        Ok(Self {
            config,
            theme,
            file_name,
            buffer,
            highlighter,
            stdout,
            gutter_width,
            offset: Offset { top: 0, left: 0 },
            cursor: Point { row: 0, column: 0 },
            mode: Mode::Normal,
            size: (width, height),
            waiting_key_command: None,
            waiting_key_action: None,
            undo: UndoStack::default(),
            command_center: CommandCenter::default(),
            last_error: None,
        })
    }

    pub fn new(config: Config, theme: Theme, file_name: Option<String>) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let buffer = if let Some(file) = &file_name {
            let content = std::fs::read_to_string(file).unwrap_or_default();
            Buffer::from_str(&content)
        } else {
            Buffer::default()
        };

        Self::new_with_size(width, height, config, theme, file_name, buffer)
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
            self.last_error = None;
            if let Some(key_action) = self.handle_event(&event) {
                log!("Key action = {key_action:?}");
                let quit = match key_action {
                    KeyAction::Single(action) => self.execute(&action, &mut buffer)?,
                    KeyAction::Multiple(actions) => {
                        let mut quit = false;
                        for action in actions {
                            quit |= self.execute(&action, &mut buffer)?;
                            if quit {
                                break;
                            }
                        }
                        quit
                    }
                    KeyAction::Nested(mapping) => {
                        if let Event::Key(KeyEvent {
                            code: KeyCode::Char(c),
                            ..
                        }) = event
                        {
                            self.waiting_key_command = Some(c);
                        }
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
            self.draw_command_line(&mut buffer);
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
        let col = self.cursor.column - self.offset.left;
        (row as u16, col as u16 + self.gutter_width)
    }

    fn reset_bounds(&mut self) -> bool {
        // Reset the column
        let mut changed = false;
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

        if self.cursor.column < self.offset.left {
            self.offset.left = self.cursor.column;
            changed = true;
        }

        if self.cursor.column >= self.offset.left + width as usize {
            self.offset.left = self
                .cursor
                .column
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
    }

    fn draw_cursor(&mut self) -> Result<()> {
        let cursor_style = match self.waiting_key_action {
            Some(_) => cursor::SetCursorStyle::SteadyUnderScore,
            None => self.mode.set_cursor_style(),
        };
        let (row, col) = if self.mode == Mode::Command {
            (self.size.0 - 1, self.command_center.position as u16 + 1)
        } else {
            self.get_current_screen_position()
        };
        self.stdout
            .queue(cursor_style)?
            .queue(cursor::MoveTo(col, row))?;
        Ok(())
    }

    fn draw_gutter(&mut self, buffer: &mut RenderBuffer) {
        let (_, height) = self.get_viewport_size();
        for i in 0..height {
            let line_number = self.offset.top + i as usize + 1;
            let content = if line_number <= self.buffer.line_count() {
                let w = (self.gutter_width - 1) as usize;
                format!("{line_number:>w$} ")
            } else {
                " ".repeat(self.gutter_width as usize)
            };
            buffer.set_text(i as usize, 0, &content, &self.theme.gutter_style);
        }
    }

    fn draw_viewport(&mut self, buffer: &mut RenderBuffer) -> Result<()> {
        let top = self.offset.top;
        let (width, height) = self.get_viewport_size();
        let code = self.buffer.to_bytes();
        let tokens = self.highlighter.highlight(&code)?;

        let mut info_iter = tokens
            .iter()
            .filter(|info| {
                info.end_position.row >= top && info.start_position.row < top + height as usize
            })
            .map(|info| {
                let mut new_info = info.clone();
                new_info.start_position.row -= top;
                new_info.end_position.row -= top;
                new_info
            })
            .peekable();

        let mut end_row = 0;
        let mut end_col = 0;

        if let Some(info) = info_iter.peek() {
            let mut row = 0;
            let mut col = self.gutter_width as usize;
            let bytes = &code[..info.byte_range.start];
            if bytes.contains(&b'\n') {
                let lines = bytes.split(|&b| b == b'\n').skip(self.offset.top);
                for line in lines {
                    let text = from_utf8(line)?;
                    let formatted = format!("{text:<w$}", w = width as usize);
                    buffer.set_text(row, col, &formatted, &self.theme.editor_style);
                    row += 1;
                    col = self.gutter_width as usize;
                }
            }
            // Skip the first line
            else {
                buffer.set_text(row, col, from_utf8(bytes)?, &self.theme.editor_style);
            }
        };

        while let Some(info) = info_iter.next() {
            let style = self.theme.get_style(&info.scope);
            let bytes = &code[info.byte_range.start..info.byte_range.end];
            end_row = info.end_position.row;
            end_col = info.end_position.column;

            self.set_text_on_viewport(
                info.start_position.row,
                info.start_position.column,
                bytes,
                buffer,
                &style,
            )?;

            match info_iter.peek() {
                // Next highlight on the same line
                Some(next) => {
                    if info.byte_range.end <= next.byte_range.start {
                        self.set_text_on_viewport(
                            info.end_position.row,
                            info.end_position.column,
                            &code[info.byte_range.end..next.byte_range.start],
                            buffer,
                            &self.theme.editor_style,
                        )?;
                    }
                }
                // Next highlight on the next line
                None => {
                    self.set_text_on_viewport(
                        info.end_position.row,
                        info.end_position.column,
                        &code[info.byte_range.end..],
                        buffer,
                        &self.theme.editor_style,
                    )?;
                }
            }
        }

        // Fill the remaining rows
        let empty = " ".repeat(width as usize);
        while end_row < height as usize {
            buffer.set_text(
                end_row,
                end_col + self.gutter_width as usize,
                &empty,
                &self.theme.editor_style,
            );
            end_row += 1;
            end_col = 0;
        }

        Ok(())
    }

    fn set_text_on_viewport(
        &self,
        row: usize,
        col: usize,
        bytes: &[u8],
        buffer: &mut RenderBuffer,
        style: &Style,
    ) -> Result<()> {
        let gutter = self.gutter_width as usize;
        let (width, height) = self.get_viewport_size();

        let is_multilines = bytes.contains(&b'\n');
        if !is_multilines {
            buffer.set_text(row, col + gutter, from_utf8(&bytes)?, style);
        } else {
            let mut lines = bytes.split(|&c| c == b'\n');
            let mut current_row = row;
            let mut current_col = col;
            while let Some(line) = lines.next() {
                let content = from_utf8(line)?;
                let text = format!("{content:<w$}", w = width as usize);
                buffer.set_text(current_row, current_col + gutter, &text, style);
                current_row += 1;
                current_col = 0;
                if current_row >= height as usize {
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw_status_line(&mut self, buffer: &mut RenderBuffer) {
        let left = format!(" {} ", self.mode.to_name().to_uppercase());
        let right = format!(" {}:{} ", self.cursor.row + 1, self.cursor.column + 1);
        let file = format!(" {}", self.file_name.as_deref().unwrap_or("new file"));
        let center_width = self.size.0 as usize - left.len() - right.len();
        let center = format!("{file:<center_width$}");

        let outer_style = match self.mode {
            Mode::Insert => &self.theme.status_line_style.insert,
            Mode::Normal => &self.theme.status_line_style.normal,
            Mode::Command => &self.theme.status_line_style.command,
        };

        let height = self.get_viewport_size().1 as usize;
        buffer.set_text(height, 0, &left, &outer_style);
        buffer.set_text(
            height,
            left.len(),
            &center,
            &self.theme.status_line_style.inner,
        );
        buffer.set_text(height, left.len() + center_width, &right, &outer_style);
    }

    fn draw_command_line(&mut self, buffer: &mut RenderBuffer) {
        let (width, height) = self.size;
        let format = if self.mode == Mode::Command {
            let command = self.command_center.buffer.clone();
            format!(":{command:<w$}", w = width as usize)
        } else if let Some(ref error) = self.last_error {
            format!("Error: {error:<w$}", w = width as usize)
        } else if let Some(c) = self.waiting_key_command {
            format!("{c:<w$}", w = width as usize)
        } else {
            " ".repeat(width as usize)
        };
        buffer.set_text(height as usize - 1, 0, &format, &self.theme.editor_style);
    }

    fn handle_event(&mut self, event: &Event) -> Option<KeyAction> {
        if let Event::Resize(width, height) = event {
            self.size = (*width, *height);
            return None;
        }

        if let Some(mapping) = &self.waiting_key_action {
            let action = get_key_action(mapping, &event);
            self.waiting_key_command = None;
            self.waiting_key_action = None;
            return action;
        }

        match &self.mode {
            Mode::Insert => self.handle_insert_event(&event),
            Mode::Normal => self.handle_normal_event(&event),
            Mode::Command => self.handle_command_event(&event),
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

    fn handle_command_event(&mut self, event: &Event) -> Option<KeyAction> {
        let action = get_key_action(&self.config.keys.command, &event);

        if action.is_some() {
            return action;
        }

        if let Event::Key(event) = event {
            if let KeyCode::Char(c) = event.code {
                self.command_center.insert(c);
            }
        }
        None
    }

    fn execute(&mut self, action: &Action, buffer: &mut RenderBuffer) -> Result<bool> {
        match action {
            Action::Quit => {
                return Ok(true);
            }
            Action::MoveUp => self.buffer.move_up(&mut self.cursor, &self.mode),
            Action::MoveDown => self.buffer.move_down(&mut self.cursor, &self.mode),
            Action::MoveLeft => match self.mode {
                Mode::Normal | Mode::Insert => self.buffer.move_left(&mut self.cursor, &self.mode),
                Mode::Command => self.command_center.move_left(),
            },
            Action::MoveRight => match self.mode {
                Mode::Normal | Mode::Insert => self.buffer.move_right(&mut self.cursor, &self.mode),
                Mode::Command => self.command_center.move_right(),
            },
            Action::PageUp => {
                let (_, height) = self.get_viewport_size();
                self.cursor.row = self.cursor.row.saturating_sub(height as usize);
                self.cursor.column = 0;
            }
            Action::PageDown => {
                let (_, height) = self.get_viewport_size();
                self.cursor.row = self
                    .buffer
                    .line_count()
                    .saturating_sub(1)
                    .min(self.cursor.row + height as usize);
                self.cursor.column = 0;
            }
            Action::MoveToTop => {
                self.cursor.row = 0;
                self.cursor.column = 0;
            }
            Action::MoveToBottom => {
                self.cursor.row = self.buffer.line_count().saturating_sub(1);
                self.cursor.column = 0;
            }
            Action::MoveToLineStart => {
                self.buffer.move_to_line_start(&mut self.cursor);
            }
            Action::MoveToLineEnd => {
                self.buffer.move_to_line_end(&mut self.cursor, &self.mode);
            }
            Action::MoveToViewportCenter => {
                let (_, height) = self.get_viewport_size();
                self.offset.top = self.cursor.row.saturating_sub(height as usize / 2);
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
            }
            Action::GotoLine(line) => {
                let line = line.min(&self.buffer.line_count()).saturating_sub(1);
                self.cursor.row = line;
                self.cursor.column = 0;
                let (_, height) = self.get_viewport_size();
                if line < self.offset.top || line > self.offset.top + height as usize - 1 {
                    self.execute(&Action::MoveToViewportCenter, buffer)?;
                }
            }
            Action::EnterMode(mode) => {
                match (&self.mode, mode) {
                    (Mode::Insert, Mode::Normal) => {
                        self.undo.batch();
                    }
                    (Mode::Command, _) => {
                        self.command_center.reset();
                    }
                    _ => {}
                }
                self.mode = mode.to_owned();
            }
            Action::InsertCharAtCursor(char) => {
                self.buffer.insert_char(*char, &mut self.cursor);
                self.draw_viewport(buffer)?;
            }
            Action::NewLineAtCursor => {
                self.buffer.insert_char('\n', &mut self.cursor);
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
            }
            Action::DeleteCharAtCursor => {
                if let Some(char) = self.buffer.get_current_char(&self.cursor) {
                    if char != '\n' || self.mode == Mode::Insert {
                        self.buffer.delete_char(&mut self.cursor, &self.mode);
                        if char == '\n' {
                            self.draw_gutter(buffer);
                        }
                        self.draw_viewport(buffer)?;
                    }
                }
            }
            Action::BackspaceCharAtCursor => match &self.mode {
                Mode::Insert | Mode::Normal => {
                    if self.cursor.row != 0 || self.cursor.column != 0 {
                        self.execute(&Action::MoveLeft, buffer)?;
                        self.execute(&Action::DeleteCharAtCursor, buffer)?;
                    }
                }
                Mode::Command => {
                    let sucess = self.command_center.backspace();
                    if !sucess {
                        self.execute(&Action::EnterMode(Mode::Normal), buffer)?;
                    }
                }
            },
            Action::DeleteCurrentLine => {
                let _ = self.buffer.delete_current_line(&mut self.cursor);
                self.draw_gutter(buffer);
                self.draw_viewport(buffer)?;
            }
            Action::Undo => {
                if let Some(action) = self.undo.pop() {
                    return self.execute(&action, buffer);
                }
            }
            Action::ExecuteCommand => {
                match self.command_center.parse_action() {
                    std::result::Result::Ok(action) => {
                        if self.execute(&action, buffer)? {
                            return Ok(true);
                        }
                    }
                    Err(error) => {
                        self.last_error = Some(error);
                    }
                };
                self.execute(&Action::EnterMode(Mode::Normal), buffer)?;
            }
            Action::Multiple(actions) => {
                for action in actions {
                    if self.execute(&action, buffer)? {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}
