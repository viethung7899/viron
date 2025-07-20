use crate::config::Config;
use crate::core::buffer_manager::BufferManager;
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::cursor::Cursor;
use crate::core::message::MessageManager;
use crate::core::mode::Mode;
use crate::core::register::RegisterManager;
use crate::core::viewport::Viewport;
use crate::input::InputState;
use crate::service::LspService;
use crate::ui::compositor::Compositor;

// Context passed to actions when they execute
pub struct EditorContext<'a> {
    pub cursor: &'a mut Cursor,
    pub viewport: &'a mut Viewport,
    pub mode: &'a mut Mode,
    pub buffer_manager: &'a mut BufferManager,
    pub register_manager: &'a mut RegisterManager,
}

pub struct UIContext<'a> {
    pub compositor: &'a mut Compositor,
}

pub struct InputContext<'a> {
    pub command_buffer: &'a mut CommandBuffer,
    pub search_buffer: &'a mut SearchBuffer,
    pub input_state: &'a mut InputState,
}

pub struct ActionContext<'a> {
    pub editor: EditorContext<'a>,
    pub ui: UIContext<'a>,
    pub input: InputContext<'a>,
    pub message: &'a mut MessageManager,
    pub config: &'a Config,
    pub running: &'a mut bool,
    pub lsp_service: &'a mut LspService,
}