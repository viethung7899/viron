pub mod core;
mod types;
pub use types::*;
mod command_parser;

use crate::config::Config;
use crate::core::buffer_manager::BufferManager;
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::message::MessageManager;
use crate::core::mode::Mode;
use crate::core::{cursor::Cursor, viewport::Viewport};
use crate::input::InputState;
use crate::service::LspService;
use crate::ui::components::ComponentIds;
use crate::ui::compositor::Compositor;
use anyhow::Result;

pub type ActionResult = Result<()>;

// Context passed to actions when they execute
pub struct ActionContext<'a> {
    pub buffer_manager: &'a mut BufferManager,
    pub command_buffer: &'a mut CommandBuffer,
    pub search_buffer: &'a mut SearchBuffer,
    pub message: &'a mut MessageManager,
    pub config: &'a Config,

    pub cursor: &'a mut Cursor,
    pub viewport: &'a mut Viewport,
    pub mode: &'a mut Mode,
    pub running: &'a mut bool,

    pub input_state: &'a mut InputState,

    pub compositor: &'a mut Compositor,
    pub component_ids: &'a ComponentIds,

    pub lsp_service: &'a mut LspService,
}
