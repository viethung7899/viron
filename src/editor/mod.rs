mod builder;
mod core;
mod input;
mod terminal;
mod ui;

pub use builder::EditorBuilder;

use crate::actions::context::{ActionContext, EditorContext, InputContext, UIContext};
use crate::actions::core::Executable;
use crate::actions::{buffer, mode};
use crate::config::Config;
use crate::config::editor::Gutter;
use crate::constants::components::{EDITOR_VIEW, PENDING_KEYS, STATUS_LINE};
use crate::core::message::MessageManager;
use crate::core::mode::Mode;
use crate::editor::core::EditorCore;
use crate::editor::input::InputSystem;
use crate::editor::terminal::TerminalContext;
use crate::editor::ui::UISystem;
use crate::input::keys::KeyEvent as VironKeyEvent;
use crate::input::{events::InputEvent, get_default_input_action};
use crate::service::LspService;
use crate::ui::context::{
    DiagnosticRenderContext, EditorRenderContext, InputRenderContext, RenderContext,
};
use anyhow::Result;
use crossterm::QueueableCommand;
use crossterm::cursor::SetCursorStyle;
use crossterm::{cursor, event::KeyEvent};
use std::io::Write;

pub struct Editor {
    core: EditorCore,
    terminal: TerminalContext,
    ui: UISystem,
    input: InputSystem,

    message_manager: MessageManager,
    config: Config,
    lsp_service: LspService,
    running: bool,
}

impl Editor {
    pub async fn from_builder(builder: EditorBuilder) -> Result<Self> {
        let terminal = TerminalContext::new()?;
        let core = EditorCore::new(terminal.width, terminal.height);
        let input = InputSystem::new();
        let ui = UISystem::new(terminal.width, terminal.height)?;
        let config = builder.config.unwrap_or_default();

        let mut editor = Self {
            terminal,
            core,
            input,
            ui,
            message_manager: MessageManager::new(),
            config,
            lsp_service: LspService::new(),
            running: true,
        };

        if let Some(file) = builder.file {
            let action = buffer::OpenBuffer::new(file);
            editor.execute_action(&action).await?;
        } else {
            editor.core.buffer_manager.new_buffer();
        }

        Ok(editor)
    }

    pub async fn run(&mut self) -> Result<()> {
        // Main event loop
        while self.running {
            // Handle events
            self.render()?;
            match self.input.event_handler.next().await? {
                InputEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key)? {
                        self.execute_action(action.as_ref()).await?;
                        if self.input.input_state.is_empty()
                            && matches!(self.core.mode, Mode::OperationPending(_))
                        {
                            self.execute_action(&mode::EnterMode::new(Mode::Normal))
                                .await?;
                        }
                    }
                }
                InputEvent::Resize(width, height) => {
                    self.handle_resize(width as usize, height as usize)?;
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
        let editor_ctx = EditorContext {
            cursor: &mut self.core.cursor,
            viewport: &mut self.core.viewport,
            mode: &mut self.core.mode,
            buffer_manager: &mut self.core.buffer_manager,
            register_system: &mut self.core.register_system,
        };

        let ui_ctx = UIContext {
            compositor: &mut self.ui.compositor,
        };

        let input_ctx = InputContext {
            command_buffer: &mut self.input.command_buffer,
            search_buffer: &mut self.input.search_buffer,
            input_state: &mut self.input.input_state,
        };

        let mut context = ActionContext {
            editor: editor_ctx,
            ui: ui_ctx,
            input: input_ctx,
            message: &mut self.message_manager,
            config: &self.config,
            running: &mut self.running,
            lsp_service: &mut self.lsp_service,
        };
        action.execute(&mut context).await
    }

    fn render(&mut self) -> Result<()> {
        self.scroll_viewport()?;

        let document = self.core.buffer_manager.current_mut();
        let uri = document.get_uri().unwrap_or_default();

        let editor = EditorRenderContext {
            viewport: &self.core.viewport,
            document,
            cursor: &self.core.cursor,
            mode: &self.core.mode,
        };

        let input = InputRenderContext {
            command_buffer: &self.input.command_buffer,
            search_buffer: &self.input.search_buffer,
            input_state: &self.input.input_state,
        };

        let diagnostics = DiagnosticRenderContext {
            diagnostics: self.lsp_service.get_diagnostics(&uri),
            message_manager: &self.message_manager,
        };

        let mut context = RenderContext {
            editor,
            input,
            diagnostics,
            config: &self.config
        };

        self.terminal.stdout.queue(cursor::Hide)?;
        self.ui
            .compositor
            .render(&mut context, &mut self.terminal.stdout)?;

        if let Some((row, col)) = self.ui.compositor.get_cursor_position(&context) {
            let set_cursor_style = self.get_cursor_style();
            self.terminal
                .stdout
                .queue(cursor::MoveTo(col as u16, row as u16))?
                .queue(set_cursor_style)?
                .queue(cursor::Show)?;
        }

        self.terminal.stdout.flush()?;

        Ok(())
    }

    fn scroll_viewport(&mut self) -> Result<()> {
        if self
            .core
            .scroll_viewport(self.config.gutter == Gutter::None)
        {
            self.ui.mark_dirty([STATUS_LINE, EDITOR_VIEW])?;
        }
        Ok(())
    }

    fn handle_resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.terminal.resize(width, height)?;
        self.ui.resize(width, height);
        self.core.resize_viewport(width, height);
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<Option<Box<dyn Executable>>> {
        // Convert to our key event type
        let key_event = VironKeyEvent::from(key);

        let default_action = get_default_input_action(&key_event, &self.core.mode);

        if default_action.is_some() {
            return Ok(default_action);
        };

        self.input.input_state.add_key(key_event);
        self.ui.compositor.mark_visible(PENDING_KEYS, true)?;
        self.ui.compositor.mark_dirty(PENDING_KEYS)?;

        let action = self
            .input
            .input_state
            .get_executable(&self.core.mode, &self.config.keymap);
        if self.input.input_state.is_empty() {
            self.ui.compositor.mark_visible(PENDING_KEYS, false)?;
        }
        Ok(action)
    }

    fn get_cursor_style(&self) -> SetCursorStyle {
        if !self.input.input_state.is_empty() {
            return SetCursorStyle::SteadyUnderScore;
        }
        match self.core.mode {
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
        self.terminal.cleanup()?;
        tokio::spawn(async move { self.lsp_service.shutdown().await });
        Ok(())
    }
}
