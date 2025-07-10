use crate::config::get_config_dir;
use crate::core::cursor::Cursor;
use crate::core::viewport::Viewport;
use crate::core::{buffer_manager::BufferManager, message::MessageManager};
use crate::input::actions::Executable;
use crate::input::keymaps::{KeyEvent as VironKeyEvent, KeyMap, KeySequence};
use crate::input::{
    actions,
    actions::ActionContext,
    events::{EventHandler, InputEvent},
};
use crate::service::LspService;
use crate::ui::components::{
    BufferView, CommandLine, ComponentIds, Gutter, MessageArea, PendingKeys, SearchBox, StatusLine,
};
use crate::ui::compositor::Compositor;
use crate::ui::{Component, RenderContext, theme::Theme};
use crate::{
    config::Config,
    core::command::{CommandBuffer, SearchBuffer},
};
use anyhow::Result;
use crossterm::{ExecutableCommand, QueueableCommand, style};
use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
};
use serde::{Deserialize, Serialize};
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};

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
    search_buffer: SearchBuffer,

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

    // Services
    lsp_service: LspService,

    // State
    running: bool,
}

impl Editor {
    pub async fn new(file: Option<impl AsRef<Path>>) -> Result<Self> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(cursor::Hide)?
            .execute(terminal::Clear(ClearType::All))?;

        let (width, height) = terminal::size()?;

        // Create core components

        let buffer_manager = file
            .as_ref()
            .map(|file| BufferManager::new_with_file(file.as_ref()))
            .unwrap_or(BufferManager::new());

        let command_buffer = CommandBuffer::new();
        let message_manager = MessageManager::new();
        let search_buffer = SearchBuffer::new();
        let cursor = Cursor::new();
        let viewport = Viewport::new(width as usize - MIN_GUTTER_SIZE, height as usize - 2);

        // Create UI components
        let theme = Theme::default();
        let mut compositor =
            Compositor::new(width as usize, height as usize, &theme.editor_style());

        // Add components to the compositor
        let status_line_id = compositor.add_component("status_line", StatusLine, true)?;
        let gutter_id = compositor.add_component("gutter", Gutter, true)?;
        let buffer_view_id = compositor.add_component("buffer_view", BufferView, true)?;
        
        // Add invisible components
        let pending_keys_id = compositor.add_component("pending_keys", PendingKeys, false)?;
        let command_line_id = compositor.add_component("command_line", CommandLine, false)?;
        let search_box_id = compositor.add_component("search_box", SearchBox, false)?;
        let message_area_id = compositor.add_component("message_area", MessageArea, false)?;

        let component_ids = ComponentIds {
            status_line_id,
            gutter_id,
            buffer_view_id,
            pending_keys_id,
            command_line_id,
            message_area_id,
            search_box_id,
        };

        // Create input handling
        let keymap = KeyMap::new();
        let pending_keys = KeySequence::new();
        let event_handler = EventHandler::new();

        // Create LSP service
        let lsp_service = LspService::new();

        let mut editor = Self {
            width: width as usize,
            height: height as usize,

            buffer_manager,
            command_buffer,
            search_buffer,
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

            lsp_service,

            running: true,
        };

        if let Some(file) = file {
            let action = actions::OpenBuffer::new(PathBuf::from(file.as_ref()));
            editor.execute_action(&action).await?;
        };

        Ok(editor)
    }

    pub fn load_config(&mut self, config: &Config) -> Result<()> {
        self.keymap = KeyMap::load_from_config(&config.keymap)?;
        let theme_path = get_config_dir().join(format!("themes/{}.json", config.theme));
        self.theme = Theme::load_from_file(theme_path)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Main event loop
        while self.running {
            // Handle events
            self.render()?;
            match self.event_handler.next().await? {
                InputEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key) {
                        self.execute_action(action.as_ref()).await?;
                    }
                }
                InputEvent::Resize(width, height) => {
                    self.handle_resize(width as usize, height as usize)?;
                    self.post_render_cleanup()?;
                }
                InputEvent::Tick => {
                    self.handle_tick().await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn execute_action(&mut self, action: &dyn Executable) -> Result<()> {
        let mut context = ActionContext {
            mode: &mut self.mode,
            viewport: &mut self.viewport,
            buffer_manager: &mut self.buffer_manager,
            command_buffer: &mut self.command_buffer,
            search_buffer: &mut self.search_buffer,
            message: &mut self.message_manager,
            cursor: &mut self.cursor,
            running: &mut self.running,
            compositor: &mut self.compositor,
            component_ids: &mut self.component_ids,
            lsp_service: &mut self.lsp_service,
        };
        action.execute(&mut context).await
    }

    fn render(&mut self) -> Result<()> {
        let gutter_width = self.gutter_width();
        self.update_viewport_for_gutter_width(gutter_width)?;

        self.scroll_viewport()?;

        let document = self.buffer_manager.current_mut();
        let uri = document.uri().unwrap_or_default();

        let mut context = RenderContext {
            theme: &self.theme,
            cursor: &self.cursor,
            document,
            diagnostics: self.lsp_service.get_diagnostics(&uri),
            mode: &self.mode,
            viewport: &self.viewport,
            command_buffer: &self.command_buffer,
            search_buffer: &self.search_buffer,
            message_manager: &self.message_manager,
            pending_keys: &self.pending_keys,
        };

        self.stdout.queue(cursor::Hide)?;
        self.compositor.render(&mut context, &mut self.stdout)?;
        self.show_cursor()?;
        self.stdout.flush()?;

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
            Mode::Search => {
                self.show_cursor_in_search_box()?;
            }
        }
        Ok(())
    }

    fn show_cursor_in_buffer(&mut self) -> Result<()> {
        let (row, column) = self.cursor.get_display_cursor();
        let viewport = &self.viewport;
        let gutter_size = self.gutter_width();

        let screen_row = row - viewport.top_line();
        let screen_col = column - viewport.left_column();

        if screen_row < viewport.height() && screen_col < viewport.width() {
            self.stdout
                .queue(cursor::MoveTo(
                    (screen_col + gutter_size) as u16,
                    screen_row as u16,
                ))?
                .queue(cursor::Show)?;
        }
        Ok(())
    }

    fn show_cursor_in_command_line(&mut self) -> Result<()> {
        let position = self.command_buffer.cursor_position();
        self.stdout
            .queue(cursor::MoveTo(position as u16 + 1, self.height as u16 - 1))?
            .queue(cursor::Show)?;
        Ok(())
    }

    fn show_cursor_in_search_box(&mut self) -> Result<()> {
        let position = self.search_buffer.buffer.cursor_position();
        self.stdout
            .queue(cursor::MoveTo(position as u16 + 1, self.height as u16 - 1))?
            .queue(cursor::Show)?;
        Ok(())
    }

    fn handle_resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.width = width;
        self.height = height;
        self.compositor.resize(width, height);
        self.viewport
            .resize(width - self.gutter_width(), height - 2);
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Box<dyn Executable>> {
        // Convert to our key event type
        let key_event = VironKeyEvent::from(key);
        self.pending_keys.add(key_event.clone());
        let action = self
            .keymap
            .get_action(&self.mode, &self.pending_keys)
            .cloned();

        if let Some(action) = &action {
            self.pending_keys.clear();
            self.compositor
                .mark_visible(&self.component_ids.pending_keys_id, false)
                .ok();
            return Some(action.clone());
        }

        if self.keymap.is_partial_match(&self.mode, &self.pending_keys) {
            self.compositor
                .mark_visible(&self.component_ids.pending_keys_id, true)
                .ok();
            return None;
        }

        self.pending_keys.clear();
        self.compositor
            .mark_visible(&self.component_ids.pending_keys_id, false)
            .ok();
        match &self.mode {
            Mode::Insert => self.handle_default_insert_event(&key_event),
            Mode::Command => self.handle_default_command_event(&key_event),
            Mode::Search => self.handle_default_search_event(&key_event),
            _ => None,
        }
    }

    async fn handle_tick(&mut self) -> Result<()> {
        let actions = self.lsp_service.handle_message().await;
        if let Some(action) = actions {
            self.execute_action(action.as_ref()).await?;
        }
        Ok(())
    }

    fn handle_default_insert_event(
        &mut self,
        key_event: &VironKeyEvent,
    ) -> Option<Box<dyn Executable>> {
        let code = key_event.code;
        let modifiers = key_event.modifiers;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                Some(Box::new(actions::InsertChar::new(ch)))
            }
            _ => None,
        }
    }

    fn handle_default_command_event(
        &mut self,
        key_event: &VironKeyEvent,
    ) -> Option<Box<dyn Executable>> {
        let code = key_event.code;
        let modifiers = key_event.modifiers;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                Some(Box::new(actions::CommandInsertChar::new(ch)))
            }
            (KeyCode::Enter, _) => Some(Box::new(actions::CommandExecute)),
            (KeyCode::Left, _) => Some(Box::new(actions::CommandMoveLeft)),
            (KeyCode::Right, _) => Some(Box::new(actions::CommandMoveLeft)),
            (KeyCode::Backspace, _) => Some(Box::new(actions::CommandBackspace)),
            (KeyCode::Delete, _) => Some(Box::new(actions::CommandDeleteChar)),
            _ => None,
        }
    }

    fn handle_default_search_event(
        &mut self,
        key_event: &VironKeyEvent,
    ) -> Option<Box<dyn Executable>> {
        let code = key_event.code;
        let modifiers = key_event.modifiers;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                Some(Box::new(actions::SearchInsertChar::new(ch)))
            }
            (KeyCode::Enter, _) => Some(Box::new(actions::SearchSubmit::new(
                self.search_buffer.buffer.content(),
            ))),
            (KeyCode::Left, _) => Some(Box::new(actions::SearchMoveLeft)),
            (KeyCode::Right, _) => Some(Box::new(actions::SearchMoveLeft)),
            (KeyCode::Backspace, _) => Some(Box::new(actions::SearchBackspace)),
            (KeyCode::Delete, _) => Some(Box::new(actions::SearchDeleteChar)),
            _ => None,
        }
    }

    pub async fn cleanup(mut self) -> Result<()> {
        // Restore terminal state
        self.stdout
            .execute(style::ResetColor)?
            .execute(cursor::Show)?
            .execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        tokio::spawn(async move { self.lsp_service.shutdown().await });

        Ok(())
    }

    fn gutter_width(&self) -> usize {
        let line_count = self.buffer_manager.current_buffer().line_count();

        let digits = line_count.to_string().len();
        (digits + 1).max(MIN_GUTTER_SIZE)
    }

    fn update_viewport_for_gutter_width(&mut self, gutter_size: usize) -> Result<()> {
        let (width, height) = terminal::size()?;
        let required_viewport_width = width as usize - gutter_size;

        // Only update if the width actually changed
        if self.viewport.width() != required_viewport_width {
            self.viewport
                .resize(required_viewport_width, height as usize - 2);
            self.compositor.mark_all_dirty();
        }
        Ok(())
    }
}
