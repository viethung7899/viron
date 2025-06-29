use anyhow::Result;
use crossterm::{QueueableCommand, queue, style};
use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use serde::{Deserialize, Serialize};
use std::io::{self, Stdout, Write};

use crate::config::Config;
use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::viewport::Viewport;
use crate::input::actions::Action;
use crate::input::keymaps::{KeyEvent as VironKeyEvent, KeyMap, KeySequence};
use crate::input::{
    actions,
    actions::ActionContext,
    events::{EventHandler, InputEvent},
};
use crate::ui::components::{BufferView, ComponentIds, Gutter, StatusLine};
use crate::ui::compositor::Compositor;
use crate::ui::{RenderContext, theme::Theme};

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

const GUTTER_SIZE: usize = 4;

pub struct Editor {
    // Core components
    buffer_manager: BufferManager,
    cursor: Cursor,
    viewport: Viewport,
    mode: Mode,
    stdout: Stdout,

    // UI Components
    compositor: Compositor,
    gutter_size: usize,
    theme: Theme,
    component_ids: ComponentIds,

    // Input handling
    keymap: KeyMap,
    pending_keys: KeySequence,
    event_handler: EventHandler,

    // State
    running: bool,
    command_buffer: String,
    search_buffer: String,
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
        let viewport = Viewport::new(height as usize - 2, width as usize - GUTTER_SIZE);

        // Create UI components
        let theme = Theme::default();
        let mut compositor =
            Compositor::new(width as usize, height as usize, &theme.editor_style());
        let status_line_id = compositor.add_component(Box::new(StatusLine::new()))?;
        let gutter_id = compositor.add_component(Box::new(Gutter::with_size(GUTTER_SIZE)))?;
        let buffer_view_id = compositor.add_component(Box::new(BufferView::new()))?;
        let component_ids = ComponentIds {
            status_line_id,
            gutter_id,
            buffer_view_id,
        };

        // Create input handling
        let keymap = KeyMap::new();
        let pending_keys = KeySequence::new();
        let event_handler = EventHandler::new();

        Ok(Self {
            buffer_manager,
            cursor,
            viewport,
            mode: Mode::Normal,
            stdout,

            theme,
            gutter_size: GUTTER_SIZE,
            compositor,
            component_ids,

            keymap,
            pending_keys,
            event_handler,

            running: true,
            command_buffer: String::new(),
            search_buffer: String::new(),
        })
    }

    pub fn load_config(&mut self, config: &Config) -> Result<()> {
        self.keymap = KeyMap::load_from_config(&config.keymap)?;
        self.theme = Theme::load_from_file(&config.theme)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        // Main event loop
        while self.running {
            self.scroll_viewport()?;
            let mut context = RenderContext {
                theme: &self.theme,
                cursor: &self.cursor,
                buffer_manager: &mut self.buffer_manager,
                mode: &self.mode,
                viewport: &self.viewport,
            };

            self.stdout.queue(cursor::Hide)?;
            self.compositor.render(&mut context, &mut self.stdout)?;

            self.show_cursor()?;

            self.stdout.flush()?;

            // Handle events
            match self.event_handler.next()? {
                InputEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key) {
                        action.execute(&mut ActionContext {
                            mode: &mut self.mode,
                            viewport: &mut self.viewport,
                            buffer_manager: &mut self.buffer_manager,
                            cursor: &mut self.cursor,
                            running: &mut self.running,
                            compositor: &mut self.compositor,
                            component_ids: &mut self.component_ids,
                        })?
                    }
                }
                InputEvent::Resize(width, height) => {
                    self.handle_resize(width as usize, height as usize)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn scroll_viewport(&mut self) -> Result<()> {
        if self.viewport.scroll_to_cursor(&self.cursor) {
            self.compositor.mark_dirty(&self.component_ids.buffer_view_id)?;
            self.compositor.mark_dirty(&self.component_ids.gutter_id)?;
            self.compositor.mark_dirty(&self.component_ids.status_line_id)?;
        }
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        let cursor = self.cursor.get_position();
        let viewport = &self.viewport;

        let screen_row = cursor.row - viewport.top_line();
        let screen_col = cursor.column - viewport.left_column() + self.gutter_size;

        if screen_row < viewport.height() && screen_col < viewport.width() {
            queue!(
                self.stdout,
                self.mode.set_cursor_style(),
                cursor::MoveTo(screen_col as u16, screen_row as u16),
                cursor::Show,
            )?;
        } else {
            queue!(self.stdout, cursor::Hide)?;
        }
        Ok(())
    }

    fn handle_resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.compositor.resize(width, height);
        self.viewport.resize(height - 2, width - self.gutter_size);
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Box<dyn Action>> {
        // Convert to our key event type
        let key_event = VironKeyEvent::from(key);
        self.pending_keys.add(key_event.clone());
        let action = self
            .keymap
            .get_action(&self.mode, &self.pending_keys)
            .cloned();

        if action.is_some() {
            self.pending_keys.clear();
            return action;
        }

        if self.keymap.is_partial_match(&self.mode, &self.pending_keys) {
            return None;
        }

        self.pending_keys.clear();
        match &self.mode {
            Mode::Insert => self.handle_default_insert_event(&key_event),
            _ => None,
        }
    }

    fn handle_default_insert_event(
        &mut self,
        key_event: &VironKeyEvent,
    ) -> Option<Box<dyn Action>> {
        let code = key_event.code;
        let modifiers = key_event.modifiers;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE) => Some(actions::insert_char(ch)),
            _ => None,
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // Restore terminal state
        execute!(
            self.stdout,
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
