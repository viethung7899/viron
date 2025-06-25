use anyhow::Result;
use crossterm::style;
use std::io::{self, Stdout, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    terminal::{self, ClearType},
};

use serde::{Deserialize, Serialize};

use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::viewport::Viewport;
use crate::input::keymaps::{KeyEvent as VironKeyEvent, KeyMap, KeySequence};
use crate::input::{
    actions::ActionContext,
    events::{EventHandler, InputEvent},
};
use crate::ui::{
    command_line::CommandLine,
    gutter::Gutter,
    message::{MessageArea, MessageType},
    renderer::Renderer,
    status_line::StatusLine,
    theme::Theme,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Mode::Normal => "normal".to_string(),
            Mode::Insert => "insert".to_string(),
            Mode::Command => "command".to_string(),
            Mode::Search => "search".to_string(),
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command | Mode::Search => "Command",
        }
    }

    pub fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
            _ => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}

pub struct Editor {
    // Core components
    buffer_manager: BufferManager,
    cursor: Cursor,
    viewport: Viewport,
    mode: Mode,

    // UI components
    renderer: Renderer<Stdout>,
    status_line: StatusLine,
    command_line: CommandLine,
    gutter: Gutter,
    message_area: MessageArea,
    theme: Theme,

    // Input handling
    keymap: KeyMap,
    pending_keys: KeySequence,
    event_handler: EventHandler,

    // State
    running: bool,
    command_buffer: String,
    search_buffer: String,
    last_tick: Instant,
    tick_rate: Duration,

    // Configuration
    show_line_numbers: bool,
}

impl Editor {
    pub fn new() -> Result<Self> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All)
        )?;

        let (width, height) = terminal::size()?;

        // Create core components
        let buffer_manager = BufferManager::new();
        let cursor = Cursor::new();
        let viewport = Viewport::new(height as usize - 2, width as usize);

        // Create UI components
        let renderer = Renderer::new(stdout)?;
        let status_line = StatusLine::new(width as usize);
        let command_line = CommandLine::new(width as usize);
        let message_area = MessageArea::new(width as usize);
        let gutter = Gutter::new();
        let theme = Theme::default_dark();

        // Create input handling
        let keymap = KeyMap::new();
        let pending_keys = KeySequence::new();
        let event_handler = EventHandler::new(Duration::from_millis(100));

        Ok(Self {
            buffer_manager,
            cursor,
            viewport,
            mode: Mode::Normal,

            renderer,
            status_line,
            command_line,
            message_area,
            gutter,
            theme,
            keymap,
            pending_keys,
            event_handler,

            running: true,
            command_buffer: String::new(),
            search_buffer: String::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(100),

            show_line_numbers: true,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Main event loop
        while self.running {
            // Handle events
            match self.event_handler.next()? {
                InputEvent::Key(key) => {
                    self.handle_key(key)?;
                }
                InputEvent::Resize(width, height) => {
                    self.handle_resize(width, height)?;
                }
                InputEvent::Tick => {
                    self.render()?;
                }
                _ => {}
            }

            // Handle ticks for animations or background processes
            let now = Instant::now();
            if now.duration_since(self.last_tick) >= self.tick_rate {
                self.last_tick = now;
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Convert to our key event type
        let key_event = VironKeyEvent::from(key);

        // Special handling for command and search modes
        match self.mode {
            Mode::Command => {
                if key.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                    self.command_line.clear();
                    return Ok(());
                } else if key.code == KeyCode::Enter {
                    self.execute_command(&self.command_line.get_command())?;
                    self.command_line.clear();
                    self.mode = Mode::Normal;
                    return Ok(());
                } else {
                    // Handle normal command line editing
                    match key.code {
                        KeyCode::Backspace => self.command_line.backspace(),
                        KeyCode::Char(c) => self.command_line.insert(c),
                        _ => {}
                    }
                    return Ok(());
                }
            }
            Mode::Search => {
                if key.code == KeyCode::Esc {
                    self.mode = Mode::Normal;
                    self.command_line.clear();
                    return Ok(());
                } else if key.code == KeyCode::Enter {
                    self.search(&self.command_line.get_command())?;
                    self.command_line.clear();
                    self.mode = Mode::Normal;
                    return Ok(());
                } else {
                    // Handle normal search line editing
                    match key.code {
                        KeyCode::Backspace => self.command_line.backspace(),
                        KeyCode::Char(c) => self.command_line.insert(c),
                        _ => {}
                    }
                    return Ok(());
                }
            }
            _ => {}
        }

        // Add key to pending sequence
        self.pending_keys.add(key_event);

        // Check if we have an action for this sequence
        if let Some(action) = self.keymap.get_action(&self.mode, &self.pending_keys) {
            let mut context = ActionContext {
                buffer_manager: &mut self.buffer_manager,
                cursor: &mut self.cursor,
                viewport: &mut self.viewport,
                mode: &mut self.mode,
            };

            if let Err(e) = action.execute(&mut context) {
                self.show_error(&format!("Action failed: {}", e));
            }
            self.pending_keys.clear();
        } else if !self.keymap.is_partial_match(&self.mode, &self.pending_keys) {
            // No matching action and not a prefix of a longer sequence
            self.pending_keys.clear();

            // For insert mode, default to inserting the character
            if let KeyCode::Char(c) = key.code {
                if self.mode == Mode::Insert && key.modifiers == KeyModifiers::NONE {
                    let position = self
                        .buffer_manager
                        .current_buffer()
                        .cursor_position(&self.cursor.get_position());
                    let new_position = self
                        .buffer_manager
                        .current_buffer_mut()
                        .insert_char(position, c);
                    self.cursor.set_position(
                        self.buffer_manager
                            .current_buffer()
                            .point_at_position(new_position),
                    );
                }
            }
        }

        Ok(())
    }

    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        // Update component dimensions
        self.viewport.resize(height as usize - 2, width as usize);
        self.status_line = StatusLine::new(width as usize);
        self.command_line = CommandLine::new(width as usize);
        // self.message_area = MessageArea::new(width as usize);
        self.renderer.resize(width as usize, height as usize);

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // Make sure cursor is visible
        self.viewport
            .scroll_to_cursor(&self.cursor, self.buffer_manager.current_buffer());

        // Clear screen
        self.renderer.clear()?;

        // Calculate offset for gutter
        let gutter_width = if self.show_line_numbers {
            self.gutter.width()
        } else {
            0
        };

        // Render gutter if enabled
        if self.show_line_numbers {
            self.gutter.render(
                self.renderer.writer(),
                self.buffer_manager.current_buffer(),
                &self.viewport,
            )?;
        }

        // Render buffer content
        self.renderer
            .render_buffer(self.buffer_manager.current_buffer_mut(), &self.viewport)?;

        let screen_height = self.renderer.height();

        // Render status line at bottom
        self.status_line.render(
            self.renderer.writer(),
            self.buffer_manager.current(),
            &self.mode,
            &self.cursor.get_position(),
            screen_height,
        )?;

        // Render message area if there's a message
        if self.message_area.has_message() {
            self.message_area
                .render(self.renderer.writer(), screen_height)?;
        }

        // If we're in command or search mode, render the command line
        if self.mode == Mode::Command || self.mode == Mode::Search {
            self.command_line
                .render(self.renderer.writer(), screen_height)?;
        }

        // Position cursor
        if let Some((row, col)) = self
            .viewport
            .buffer_to_viewport(&self.cursor.get_position())
        {
            let screen_row = row as u16;
            let screen_col = (col + gutter_width) as u16; // Add gutter width
            queue!(
                self.renderer.writer(),
                cursor::MoveTo(screen_col, screen_row),
                self.mode.set_cursor_style(),
                cursor::Show
            )?;
        }

        // Flush all output
        self.renderer.flush()?;

        Ok(())
    }

    fn execute_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "q" | "quit" => {
                self.running = false;
            }
            "w" | "write" => {
                if parts.len() > 1 {
                    // Save as specified path
                    let path = parts[1];
                    match self.buffer_manager.save_current_as(Path::new(path)) {
                        Ok(msg) => self.show_message(&msg),
                        Err(e) => self.show_error(&format!("Failed to save: {}", e)),
                    }
                } else {
                    // Save current file
                    match self.buffer_manager.save_current() {
                        Ok(msg) => self.show_message(&msg),
                        Err(e) => self.show_error(&format!("Failed to save: {}", e)),
                    }
                }
            }
            "e" | "edit" => {
                if parts.len() > 1 {
                    // Open specified file
                    let path = parts[1];
                    match self.buffer_manager.open_file(Path::new(path)) {
                        Ok(_) => self.show_message(&format!("Opened {}", path)),
                        Err(e) => self.show_error(&format!("Failed to open: {}", e)),
                    }
                }
            }
            "set" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "number" | "nu" => {
                            self.show_line_numbers = true;
                            self.show_message("Line numbers enabled");
                        }
                        "nonumber" | "nonu" => {
                            self.show_line_numbers = false;
                            self.show_message("Line numbers disabled");
                        }
                        _ => {
                            self.show_error(&format!("Unknown option: {}", parts[1]));
                        }
                    }
                }
            }
            _ => {
                self.show_error(&format!("Unknown command: {}", parts[0]));
            }
        }

        Ok(())
    }

    fn search(&mut self, pattern: &str) -> Result<()> {
        // Implementation of search function
        self.show_message(&format!("Searching for: {}", pattern));
        Ok(())
    }

    fn show_message(&mut self, message: &str) {
        self.message_area.set_message(message, MessageType::Info);
    }

    fn show_error(&mut self, message: &str) {
        self.message_area.set_message(message, MessageType::Error);
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // Restore terminal state
        execute!(
            self.renderer.writer(),
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    pub fn buffer_manager_mut(&mut self) -> &mut BufferManager {
        &mut self.buffer_manager
    }
}
