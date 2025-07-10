use crate::config::editor::Gutter;
use crate::config::Config;
use crate::constants::{MIN_GUTTER_WIDTH, RESERVED_ROW_COUNT};
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::cursor::Cursor;
use crate::core::mode::Mode;
use crate::core::viewport::Viewport;
use crate::core::{buffer_manager::BufferManager, message::MessageManager};
use crate::input::actions::{EnterMode, Executable};
use crate::input::keys::KeyEvent as VironKeyEvent;
use crate::input::{
    actions, actions::ActionContext,
    events::{EventHandler, InputEvent},
    get_default_command_action,
    get_default_insert_action, get_default_search_action, InputState,
};
use crate::service::LspService;
use crate::ui::components::{
    CommandLine, ComponentIds, EditorView, MessageArea, PendingKeys, SearchBox, StatusLine,
};
use crate::ui::compositor::Compositor;
use crate::ui::{theme::Theme, RenderContext};
use crate::ui::{Component, RenderContext, theme::Theme};
use crate::{
    config::Config,
    core::command::{CommandBuffer, SearchBuffer},
};
use anyhow::Result;
use crossterm::cursor::SetCursorStyle;
use crossterm::{ExecutableCommand, QueueableCommand, style};
use crossterm::{
    cursor,
    event::KeyEvent,
    terminal::{self, ClearType},
};
use crossterm::{style, ExecutableCommand, QueueableCommand};
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};

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

    // Config
    config: Config,

    // UI Components
    compositor: Compositor,
    component_ids: ComponentIds,

    // Input handling
    input_state: InputState,
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
        let buffer_manager = BufferManager::new();
        let command_buffer = CommandBuffer::new();
        let message_manager = MessageManager::new();
        let search_buffer = SearchBuffer::new();
        let cursor = Cursor::new();
        let viewport = Viewport::new(width as usize, height as usize - RESERVED_ROW_COUNT);

        // Create UI components
        let theme = Theme::default();
        let mut compositor =
            Compositor::new(width as usize, height as usize, &theme.editor_style());

        // Add components to the compositor
        let status_line_id = compositor.add_component("status_line", StatusLine, true)?;
        let editor_view_id =
            compositor.add_focusable_component("editor_view", EditorView::new(), true)?;

        // Add invisible components
        let pending_keys_id = compositor.add_component("pending_keys", PendingKeys, false)?;
        let command_line_id =
            compositor.add_focusable_component("command_line", CommandLine, false)?;
        let search_box_id = compositor.add_focusable_component("search_box", SearchBox, false)?;
        let message_area_id = compositor.add_component("message_area", MessageArea, false)?;

        compositor.set_focus(&editor_view_id)?;

        let component_ids = ComponentIds {
            status_line_id,
            editor_view_id,
            pending_keys_id,
            command_line_id,
            message_area_id,
            search_box_id,
        };

        // Create input handling
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

            config: Config::default(),

            compositor,
            component_ids,

            input_state: InputState::new(),
            event_handler,

            lsp_service,

            running: true,
        };

        if let Some(file) = file {
            let action = actions::OpenBuffer::new(PathBuf::from(file.as_ref()));
            editor.execute_action(&action).await?;
        } else {
            editor.buffer_manager.new_buffer();
        }

        Ok(editor)
    }

    pub fn load_config(&mut self, config: impl AsRef<Path>) -> Result<()> {
        self.config = Config::load_from_file(config)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Main event loop
        while self.running {
            // Handle events
            self.render()?;
            match self.event_handler.next().await? {
                InputEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key)? {
                        self.execute_action(action.as_ref()).await?;
                        if self.input_state.is_empty() && matches!(self.mode, Mode::OperationPending(_)) {
                            self.execute_action(&EnterMode::new(Mode::Normal)).await?;
                        }
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
            config: &self.config,
            message: &mut self.message_manager,
            cursor: &mut self.cursor,
            running: &mut self.running,
            compositor: &mut self.compositor,
            component_ids: &mut self.component_ids,
            lsp_service: &mut self.lsp_service,
            input_state: &mut self.input_state,
        };
        action.execute(&mut context).await
    }

    fn render(&mut self) -> Result<()> {
        self.scroll_viewport()?;

        let document = self.buffer_manager.current_mut();
        let uri = document.uri().unwrap_or_default();

        let mut context = RenderContext {
            config: &self.config,
            cursor: &self.cursor,
            document,
            diagnostics: self.lsp_service.get_diagnostics(&uri),
            mode: &self.mode,
            viewport: &self.viewport,
            command_buffer: &self.command_buffer,
            search_buffer: &self.search_buffer,
            message_manager: &self.message_manager,
            input_state: &self.input_state,
        };

        self.stdout.queue(cursor::Hide)?;
        self.compositor.render(&mut context, &mut self.stdout)?;

        if let Some((row, col)) = self.compositor.get_cursor_position(&context) {
            let set_cursor_style = self.get_cursor_style();
            self.stdout
                .queue(cursor::MoveTo(col as u16, row as u16))?
                .queue(set_cursor_style)?
                .queue(cursor::Show)?;
        }

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
        let line_count = self.buffer_manager.current_buffer().line_count();
        let gutter_width = if self.config.gutter == Gutter::None {
            0
        } else {
            (line_count.to_string().len() + 1).max(MIN_GUTTER_WIDTH)
        };
        if self
            .viewport
            .scroll_to_cursor_with_gutter(&self.cursor, gutter_width)
        {
            self.compositor
                .mark_dirty(&self.component_ids.editor_view_id)?;
            self.compositor
                .mark_dirty(&self.component_ids.status_line_id)?;
        }
        Ok(())
    }

    fn handle_resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.width = width;
        self.height = height;
        self.compositor.resize(width, height);
        self.viewport.resize(width, height - RESERVED_ROW_COUNT);
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<Option<Box<dyn Executable>>> {
        // Convert to our key event type
        let key_event = VironKeyEvent::from(key);

        let default_action = match &self.mode {
            Mode::Insert => get_default_insert_action(&key_event),
            Mode::Command => get_default_command_action(&key_event),
            Mode::Search => get_default_search_action(&key_event),
            _ => None,
        };

        if default_action.is_some() {
            return Ok(default_action);
        };

        self.input_state.add_key(key_event);
        self.compositor
            .mark_visible(&self.component_ids.pending_keys_id, true)?;
        self.compositor
            .mark_dirty(&self.component_ids.pending_keys_id)?;

        let action = self
            .input_state
            .get_executable(&self.mode, &self.config.keymap);
        if self.input_state.is_empty() {
            self.compositor
                .mark_visible(&self.component_ids.pending_keys_id, false)?;
        }
        Ok(action)
    }

    fn get_cursor_style(&self) -> SetCursorStyle {
        if !self.input_state.is_empty() {
            return SetCursorStyle::SteadyUnderScore;
        }
        match self.mode {
            Mode::Normal => SetCursorStyle::DefaultUserShape,
            Mode::Insert | Mode::Command | Mode::Search => SetCursorStyle::BlinkingBar,
            Mode::OperationPending(_) => SetCursorStyle::SteadyUnderScore,
        }
    }

    async fn handle_tick(&mut self) -> Result<()> {
        let Some(client) = self.lsp_service.get_client_mut() else {
            return Ok(());
        };
        if let Some(action) = client.get_lsp_action().await? {
            self.execute_action(action.as_ref()).await?;
        };
        Ok(())
    }

    pub async fn cleanup(mut self) -> Result<()> {
        // Restore terminal state
        self.stdout
            .execute(style::ResetColor)?
            .execute(cursor::Show)?
            .execute(SetCursorStyle::DefaultUserShape)?
            .execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        tokio::spawn(async move { self.lsp_service.shutdown().await });

        Ok(())
    }
}
