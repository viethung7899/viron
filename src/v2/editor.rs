use anyhow::Result;
use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use crossterm::{queue, style, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::io::{self, Stdout, Write};

use crate::core::cursor::Cursor;
use crate::core::viewport::Viewport;
use crate::core::{buffer_manager::BufferManager, message::MessageManager};
use crate::input::actions::Action;
use crate::input::keymaps::{KeyEvent as VironKeyEvent, KeyMap, KeySequence};
use crate::input::{
    actions,
    actions::ActionContext,
    events::{EventHandler, InputEvent},
};
use crate::ui::components::{
    BufferView, CommandLine, ComponentIds, Gutter, MessageArea, StatusLine,
};
use crate::ui::compositor::Compositor;
use crate::ui::{theme::Theme, Component, RenderContext};
use crate::{config::Config, core::command_buffer::CommandBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            Mode::Command => "Command",
            Mode::Search => "Search",
        }
    }

    pub fn set_cursor_style(&self) -> cursor::SetCursorStyle {
        match self {
            Mode::Insert => cursor::SetCursorStyle::SteadyBar,
            _ => cursor::SetCursorStyle::SteadyBlock,
        }
    }
}

const MIN_GUTTER_SIZE: usize = 4;

pub struct Editor {
    // Size
    width: usize,
    height: usize,

    // Core components
    buffer_manager: BufferManager,
    command_buffer: CommandBuffer,
    message_manager: MessageManager,
    cursor: Cursor,
    viewport: Viewport,
    mode: Mode,
    stdout: Stdout,

    // UI Components
    compositor: Compositor,
    theme: Theme,
    component_ids: ComponentIds,

    // Input handling
    keymap: KeyMap,
    pending_keys: KeySequence,
    event_handler: EventHandler,

    // State
    running: bool,
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
        let command_buffer = CommandBuffer::new();
        let message_manager = MessageManager::new();
        let cursor = Cursor::new();
        let viewport = Viewport::new(height as usize - 2, width as usize - MIN_GUTTER_SIZE);

        // Create UI components
        let theme = Theme::default();
        let mut compositor =
            Compositor::new(width as usize, height as usize, &theme.editor_style());

        let status_line_id =
            compositor.add_component(Component::new("status_line", Box::new(StatusLine)))?;
        let gutter_id = compositor.add_component(Component::new("gutter", Box::new(Gutter)))?;
        let buffer_view_id =
            compositor.add_component(Component::new("buffer_view", Box::new(BufferView)))?;
        let mut command_line = Component::new("command_line", Box::new(CommandLine));
        command_line.visible = false;
        let command_line_id = compositor.add_component(command_line)?;
        let mut message_area = Component::new("message_area", Box::new(MessageArea));
        message_area.visible = false;
        let message_area_id =
            compositor.add_component(message_area)?;

        let component_ids = ComponentIds {
            status_line_id,
            gutter_id,
            buffer_view_id,
            command_line_id,
            message_area_id,
        };

        // Create input handling
        let keymap = KeyMap::new();
        let pending_keys = KeySequence::new();
        let event_handler = EventHandler::new();

        Ok(Self {
            width: width as usize,
            height: height as usize,

            buffer_manager,
            command_buffer,
            message_manager,
            cursor,
            viewport,
            mode: Mode::Normal,
            stdout,

            theme,
            compositor,
            component_ids,

            keymap,
            pending_keys,
            event_handler,

            running: true,
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
            let gutter_width = self.gutter_width();
            self.update_viewport_for_gutter_width(gutter_width)?;

            self.scroll_viewport()?;

            let mut context = RenderContext {
                theme: &self.theme,
                cursor: &self.cursor,
                buffer_manager: &mut self.buffer_manager,
                mode: &self.mode,
                viewport: &self.viewport,
                command_buffer: &self.command_buffer,
                message_manager: &self.message_manager,
                gutter_width,
            };

            self.stdout.queue(cursor::Hide)?;
            self.compositor.render(&mut context, &mut self.stdout)?;
            self.show_cursor()?;
            self.stdout.flush()?;

            // Clean up messages after rendering
            self.post_render_cleanup()?;

            // Handle events
            match self.event_handler.next()? {
                InputEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key) {
                        Action::execute(
                            action.as_ref(),
                            &mut ActionContext {
                                mode: &mut self.mode,
                                viewport: &mut self.viewport,
                                buffer_manager: &mut self.buffer_manager,
                                command_buffer: &mut self.command_buffer,
                                message: &mut self.message_manager,
                                cursor: &mut self.cursor,
                                running: &mut self.running,
                                compositor: &mut self.compositor,
                                component_ids: &mut self.component_ids,
                            },
                        )?
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

    fn post_render_cleanup(&mut self) -> Result<()> {
        // Clear the message area after rendering
        self.message_manager.clear_message();
        self.compositor
            .mark_visible(&self.component_ids.message_area_id, false)?;
        Ok(())
    }

    fn scroll_viewport(&mut self) -> Result<()> {
        if self.viewport.scroll_to_cursor(&self.cursor) {
            self.compositor
                .mark_dirty(&self.component_ids.buffer_view_id)?;
            self.compositor.mark_dirty(&self.component_ids.gutter_id)?;
            self.compositor
                .mark_dirty(&self.component_ids.status_line_id)?;
        }
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.stdout.queue(self.mode.set_cursor_style())?;
        match self.mode {
            Mode::Normal | Mode::Insert => {
                self.show_cursor_in_buffer()?;
            }
            Mode::Command => {
                self.show_cursor_in_command_line()?;
            }
            _ => {
                self.stdout.queue(cursor::Hide)?;
            }
        }
        Ok(())
    }

    fn show_cursor_in_buffer(&mut self) -> Result<()> {
        let cursor = self.cursor.get_position();
        let viewport = &self.viewport;
        let gutter_size = self.gutter_width();

        let screen_row = cursor.row - viewport.top_line();
        let screen_col = cursor.column - viewport.left_column();

        if screen_row < viewport.height() && screen_col < viewport.width() {
            queue!(
                self.stdout,
                cursor::MoveTo((screen_col + gutter_size) as u16, screen_row as u16),
                cursor::Show,
            )?;
        } else {
            queue!(self.stdout, cursor::Hide)?;
        }
        Ok(())
    }

    fn show_cursor_in_command_line(&mut self) -> Result<()> {
        let position = self.command_buffer.cursor_position();
        queue!(
            self.stdout,
            cursor::MoveTo(position as u16 + 1, self.height as u16 - 1),
            cursor::Show,
        )?;
        Ok(())
    }

    fn handle_resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.width = width;
        self.height = height;
        self.compositor.resize(width, height);
        self.viewport
            .resize(height - 2, width - self.gutter_width());
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
            Mode::Command => self.handle_default_command_event(&key_event),
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
            (KeyCode::Char(ch), KeyModifiers::NONE) => Some(Box::new(actions::InsertChar::new(ch))),
            (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                Some(Box::new(actions::InsertChar::new(ch)))
            }
            _ => None,
        }
    }

    fn handle_default_command_event(
        &mut self,
        key_event: &VironKeyEvent,
    ) -> Option<Box<dyn Action>> {
        let code = key_event.code;
        let modifiers = key_event.modifiers;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE) => {
                Some(Box::new(actions::CommandInsertChar::new(ch)))
            }
            (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                Some(Box::new(actions::CommandInsertChar::new(ch)))
            }
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

    fn gutter_width(&self) -> usize {
        let line_count = self.buffer_manager.current_buffer().line_count();

        // Calculate digits needed for line numbers + 1 space
        let digits = line_count.to_string().len();
        (digits + 1).max(MIN_GUTTER_SIZE)
    }

    fn update_viewport_for_gutter_width(&mut self, gutter_size: usize) -> Result<()> {
        let terminal_size = terminal::size()?;
        let required_viewport_width = terminal_size.0 as usize - gutter_size;

        // Only update if the width actually changed
        if self.viewport.width() != required_viewport_width {
            self.viewport
                .resize(terminal_size.1 as usize - 2, required_viewport_width);
            self.compositor.mark_all_dirty();
        }
        Ok(())
    }
}
