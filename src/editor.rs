mod command_center;
mod render_buffer;

use std::{
    io::{Stdout, Write, stdout},
    result::Result::Ok,
    str::from_utf8,
    time::{Duration, Instant},
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
    lsp::{InboundMessage, LspClient},
    theme::{Style, Theme},
};
use anyhow::Result;
use async_recursion::async_recursion;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, EventStream, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    style::{self, StyledContent},
    terminal::{self},
};
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;
use serde::{Deserialize, Serialize};
use tokio::select;
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

    MoveToNextWord,
    MoveToPreviousWord,

    ScrollUp,
    ScrollDown,

    GotoLine(usize),
    MoveTo(usize, usize),

    InsertCharAtCursor(char),
    InsertNewLineAtCursor,
    InsertNewLineAtCurrentLine,
    InsertTabAtCursor,

    BackspaceCharAtCursor,
    DeleteCharAtCursor,
    DeleteCurrentLine,
    DeleteWord,

    EnterMode(Mode),

    ExecuteCommand,

    Multiple(Vec<Action>),

    GotoDefinition,
    Save,
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
    lsp: LspClient,
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
    last_message: Option<String>,
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.flush();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = self.stdout.execute(event::DisableMouseCapture);
        let _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    pub fn new_with_size(
        size: (u16, u16),
        config: Config,
        theme: Theme,
        lsp: LspClient,
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
            lsp,
            stdout,
            gutter_width,
            offset: Offset { top: 0, left: 0 },
            cursor: Point { row: 0, column: 0 },
            mode: Mode::Normal,
            size,
            waiting_key_command: None,
            waiting_key_action: None,
            undo: UndoStack::default(),
            command_center: CommandCenter::default(),
            last_message: None,
        })
    }

    pub async fn new(
        config: Config,
        theme: Theme,
        mut lsp: LspClient,
        file_name: Option<String>,
    ) -> Result<Self> {
        let buffer = if let Some(file) = &file_name {
            let content = std::fs::read_to_string(file).unwrap_or_default();
            lsp.did_open(file, &content).await?;
            Buffer::from_str(&content)
        } else {
            Buffer::default()
        };

        Self::new_with_size(terminal::size()?, config, theme, lsp, file_name, buffer)
    }

    pub async fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout
            .execute(event::EnableMouseCapture)?
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let (width, height) = self.size;
        let mut buffer = RenderBuffer::new(
            width as usize,
            height as usize,
            Some(self.theme.editor_style.clone()),
        );

        self.render(&mut buffer)?;

        let mut reader = EventStream::new();

        loop {
            let delay = Delay::new(Duration::from_millis(500)).fuse();
            let event = reader.next().fuse();

            select! {
                _ = delay => {
                    // handle responses from lsp
                    if let Some((msg, method)) = self.lsp.recv_response().await? {
                        if let Some(action) = self.handle_lsp_message(&msg, method) {
                            let current_buffer = buffer.clone();
                            log!("executing action from lsp: {action:?}");
                            if self.execute(&action, &mut buffer).await? {
                                break;
                            }
                            self.rerender(&current_buffer, &mut buffer)?;
                        }

                    }
                }
                option_event = event => {
                    match option_event {
                        Some(Err(error)) => {
                            log!("error: {error}");
                        },
                        Some(std::result::Result::Ok(event)) => {
                            let current_buffer = buffer.clone();
                            self.last_message = None;
                            if let Some(key_action) = self.handle_event(&event) {
                                let quit = match key_action {
                                    KeyAction::Single(action) => self.execute(&action, &mut buffer).await?,
                                    KeyAction::Multiple(actions) => {
                                        let mut quit = false;
                                        for action in actions {
                                            quit |= self.execute(&action, &mut buffer).await?;
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
                            self.rerender(&current_buffer, &mut buffer)?;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_lsp_message(
        &mut self,
        message: &InboundMessage,
        method: Option<String>,
    ) -> Option<Action> {
        match message {
            InboundMessage::ProcessingError(error) => {
                self.last_message = Some(error.to_owned());
            }
            InboundMessage::Notification(notification) => {
                log!("got an unhandled notification: {notification:?}");
            }
            InboundMessage::Error(error_msg) => {
                log!("got an error: {error_msg:?}");
            }
            InboundMessage::Message(message) => {
                let Some(method) = method else {
                    return None;
                };
                match method.as_str() {
                    "textDocument/definition" => {
                        let result = match message.result {
                            serde_json::Value::Array(ref arr) => arr[0].as_object().unwrap(),
                            serde_json::Value::Object(ref obj) => obj,
                            _ => return None,
                        };
                        let Some(start) = result.get("range").and_then(|range| range.get("start"))
                        else {
                            return None;
                        };
                        if let (Some(line), Some(character)) = (
                            start.get("line").and_then(|line| line.as_u64()),
                            start
                                .get("character")
                                .and_then(|character| character.as_u64()),
                        ) {
                            return Some(Action::MoveTo(line as usize, character as usize));
                        }
                    }
                    _ => {}
                }
            }
        }
        None
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

    fn rerender(&mut self, current_buffer: &RenderBuffer, buffer: &mut RenderBuffer) -> Result<()> {
        self.stdout.execute(cursor::Hide)?;
        self.draw_status_line(buffer);
        self.draw_command_line(buffer);
        self.render_diff(buffer.diff(&current_buffer))?;
        self.draw_cursor()?;
        self.stdout.execute(cursor::Show)?;
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

        let mut position = Point { row: 0, column: 0 };

        let first = if let Some(info) = info_iter.peek() {
            &code[..info.byte_range.start]
        } else {
            &code
        };

        let mut lines = first
            .split(|&b| b == b'\n')
            .skip(self.offset.top)
            .peekable();

        while let Some(line) = lines.next() {
            let text = from_utf8(line)?;
            if lines.peek().is_some() {
                let formatted = format!("{text:<w$}", w = width as usize);
                buffer.set_text(
                    position.row,
                    position.column + self.gutter_width as usize,
                    &formatted,
                    &self.theme.editor_style,
                );
                if position.row + 1 >= height as usize {
                    break;
                }
                position.row += 1;
                position.column = 0;
            } else {
                buffer.set_text(
                    position.row,
                    position.column + self.gutter_width as usize,
                    text,
                    &self.theme.editor_style,
                );
                position.column += text.len();
            }
        }

        while let Some(info) = info_iter.next() {
            let style = self.theme.get_style(&info.scope);
            let bytes = &code[info.byte_range.start..info.byte_range.end];
            position.row = info.end_position.row;
            position.column = info.end_position.column;

            self.set_text_on_viewport(
                &mut Point {
                    row: info.start_position.row,
                    column: info.start_position.column,
                },
                bytes,
                buffer,
                &style,
            )?;

            match info_iter.peek() {
                // Next highlight on the same line
                Some(next) => {
                    if info.byte_range.end <= next.byte_range.start {
                        self.set_text_on_viewport(
                            &mut position,
                            &code[info.byte_range.end..next.byte_range.start],
                            buffer,
                            &self.theme.editor_style,
                        )?;
                    }
                }
                // Next highlight on the next line
                None => {
                    self.set_text_on_viewport(
                        &mut position,
                        &code[info.byte_range.end..],
                        buffer,
                        &self.theme.editor_style,
                    )?;
                }
            }
        }

        // Fill the remaining rows
        let empty = " ".repeat(width as usize);
        while position.row < height as usize {
            buffer.set_text(
                position.row,
                position.column + self.gutter_width as usize,
                &empty,
                &self.theme.editor_style,
            );
            position.row += 1;
            position.column = 0;
        }

        Ok(())
    }

    fn set_text_on_viewport(
        &self,
        position: &mut Point,
        bytes: &[u8],
        buffer: &mut RenderBuffer,
        style: &Style,
    ) -> Result<()> {
        let gutter = self.gutter_width as usize;
        let (width, height) = self.get_viewport_size();

        let mut lines = bytes.split(|&c| c == b'\n').peekable();

        while let Some(line) = lines.next() {
            let text = from_utf8(&line)?;
            if lines.peek().is_some() {
                let text = format!("{text:<w$}", w = width as usize);
                buffer.set_text(position.row, position.column + gutter, &text, style);
                if position.row + 1 >= height as usize {
                    break;
                }
                position.row += 1;
                position.column = 0;
            } else {
                buffer.set_text(position.row, position.column + gutter, text, style);
                position.column += text.len();
            }
        }
        Ok(())
    }

    fn draw_status_line(&mut self, buffer: &mut RenderBuffer) {
        let left = format!(" {} ", self.mode.to_name().to_uppercase());
        let right = format!(" {}:{} ", self.cursor.row + 1, self.cursor.column + 1);
        let dirty = if self.buffer.is_dirty() { " [+]" } else { "" };
        let file = format!(
            " {}{}",
            self.file_name.as_deref().unwrap_or("new file"),
            dirty
        );
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
        } else if let Some(ref error) = self.last_message {
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

        let action = match &self.mode {
            Mode::Insert => self.handle_insert_event(&event),
            Mode::Normal => self.handle_normal_event(&event),
            Mode::Command => self.handle_command_event(&event),
        };

        if action.is_some() {
            return action;
        }

        if let Event::Mouse(mouse_event) = event {
            return self.handle_mouse_event(mouse_event);
        }

        None
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) -> Option<KeyAction> {
        let row = mouse_event.row;
        let column = mouse_event.column;
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let (_, height) = self.get_viewport_size();
                if column < self.gutter_width || row >= height {
                    return None;
                }
                Some(KeyAction::Single(Action::MoveTo(
                    row as usize + self.offset.top,
                    (column - self.gutter_width) as usize,
                )))
            }
            MouseEventKind::ScrollUp => Some(KeyAction::Single(Action::ScrollUp)),
            MouseEventKind::ScrollDown => Some(KeyAction::Single(Action::ScrollDown)),
            _ => None,
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

    #[async_recursion]
    async fn execute(&mut self, action: &Action, buffer: &mut RenderBuffer) -> Result<bool> {
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
            Action::MoveToNextWord => {
                if let Some(point) = self.buffer.find_next_word(&self.cursor) {
                    self.execute(&Action::MoveTo(point.row, point.column), buffer)
                        .await?;
                }
            }
            Action::MoveToPreviousWord => {
                if let Some(point) = self.buffer.find_previous_word(&self.cursor) {
                    self.execute(&Action::MoveTo(point.row, point.column), buffer)
                        .await?;
                }
            }
            Action::GotoLine(line) => {
                let line = self.buffer.line_count().min(*line);
                self.cursor.row = line;
                self.cursor.column = 0;
                let (_, height) = self.get_viewport_size();
                if line < self.offset.top || line > self.offset.top + height as usize - 1 {
                    self.execute(&Action::MoveToViewportCenter, buffer).await?;
                }
            }
            Action::MoveTo(line, column) => {
                self.execute(&Action::GotoLine(*line), buffer).await?;
                self.cursor.column = *column;
                self.buffer.clamp_column(&mut self.cursor, &self.mode);
            }
            Action::ScrollUp => {
                let (_, height) = self.get_viewport_size();
                let scroll_lines = self.config.mouse_scroll_lines.unwrap_or(3);
                self.offset.top = self.offset.top.saturating_sub(scroll_lines);
                let end_row = self.offset.top + height as usize - 1;
                if self.cursor.row > end_row {
                    self.cursor.row = end_row;
                    self.buffer.clamp_column(&mut self.cursor, &self.mode);
                }
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
            }
            Action::ScrollDown => {
                let lines = self.buffer.line_count().saturating_sub(1);
                let scroll_lines = self.config.mouse_scroll_lines.unwrap_or(3);
                self.offset.top = lines.min(self.offset.top + scroll_lines);
                if self.cursor.row < self.offset.top {
                    self.cursor.row = self.offset.top;
                    self.buffer.clamp_column(&mut self.cursor, &self.mode);
                }
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
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
            Action::InsertNewLineAtCursor => {
                let content = self.buffer.get_content_line(self.cursor.row);
                log!("Before content: {content:?}");
                let leading_spaces = content.chars().take_while(|c| c.is_whitespace()).count();
                self.buffer.insert_char('\n', &mut self.cursor);
                self.buffer
                    .insert_string(&" ".repeat(leading_spaces), &mut self.cursor);
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
            }
            Action::InsertNewLineAtCurrentLine => {
                let content = self.buffer.get_content_line(self.cursor.row);
                log!("Before content: {content:?}");
                let leading_spaces = content.chars().take_while(|c| c.is_whitespace()).count();
                self.cursor.column = 0;
                self.buffer
                    .insert_string(&" ".repeat(leading_spaces), &mut self.cursor);
                self.buffer.insert_char('\n', &mut self.cursor);
                self.execute(&Action::EnterMode(Mode::Insert), buffer)
                    .await?;
                self.execute(&Action::MoveUp, buffer).await?;
                self.execute(&Action::MoveToLineEnd, buffer).await?;
                self.draw_viewport(buffer)?;
                self.draw_gutter(buffer);
            }
            Action::InsertTabAtCursor => {
                let tab = " ".repeat(self.config.tab_size);
                self.buffer.insert_string(&tab, &mut self.cursor);
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
                        self.execute(&Action::MoveLeft, buffer).await?;
                        self.execute(&Action::DeleteCharAtCursor, buffer).await?;
                    }
                }
                Mode::Command => {
                    let sucess = self.command_center.backspace();
                    if !sucess {
                        self.execute(&Action::EnterMode(Mode::Normal), buffer)
                            .await?;
                    }
                }
            },
            Action::DeleteCurrentLine => {
                let _ = self.buffer.delete_current_line(&mut self.cursor);
                while let Some(' ') = self.buffer.get_current_char(&self.cursor) {
                    self.buffer.move_right(&mut self.cursor, &self.mode);
                }
                self.draw_gutter(buffer);
                self.draw_viewport(buffer)?;
            }
            Action::DeleteWord => {
                if self.cursor.column == 0
                    && Some('\n') == self.buffer.get_current_char(&self.cursor)
                {
                    self.execute(&Action::DeleteCurrentLine, buffer).await?;
                } else {
                    self.buffer.delete_word_inline(&mut self.cursor);
                    self.draw_gutter(buffer);
                    self.draw_viewport(buffer)?;
                }
            }
            Action::Undo => {
                if let Some(action) = self.undo.pop() {
                    return self.execute(&action, buffer).await;
                }
            }
            Action::ExecuteCommand => {
                match self.command_center.parse_action() {
                    Ok(action) => {
                        if self.execute(&action, buffer).await? {
                            return Ok(true);
                        }
                    }
                    Err(error) => {
                        self.last_message = Some(error);
                    }
                };
                self.execute(&Action::EnterMode(Mode::Normal), buffer)
                    .await?;
            }
            Action::Multiple(actions) => {
                for action in actions {
                    if self.execute(&action, buffer).await? {
                        return Ok(true);
                    }
                }
            }
            Action::GotoDefinition => {
                if let Some(file) = &self.file_name {
                    log!("going to definition for {file}");
                    self.lsp
                        .goto_definition(file, self.cursor.row, self.cursor.column)
                        .await?;
                };
            }
            Action::Save => {
                if let Some(file) = &self.file_name {
                    match self.buffer.save(file) {
                        Ok(message) => {
                            self.last_message = Some(message);
                        }
                        Err(err) => {
                            self.last_message = Some(err.to_string());
                        }
                    }
                } else {
                    self.last_message = Some("No file name".into());
                }
            }
        }
        Ok(false)
    }
}
