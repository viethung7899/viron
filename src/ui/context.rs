use crate::config::Config;
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::cursor::Cursor;
use crate::core::document::Document;
use crate::core::message::MessageManager;
use crate::core::mode::Mode;
use crate::core::viewport::Viewport;
use crate::input::InputProcessor;
use lsp_types::Diagnostic;

pub struct EditorRenderContext<'a> {
    pub viewport: &'a Viewport,
    pub document: &'a mut Document,
    pub cursor: &'a Cursor,
    pub mode: &'a Mode,
}

pub struct InputRenderContext<'a> {
    pub command_buffer: &'a CommandBuffer,
    pub search_buffer: &'a SearchBuffer,
    pub input_state: &'a InputProcessor,
}

pub struct DiagnosticRenderContext<'a> {
    pub diagnostics: &'a [Diagnostic],
    pub message_manager: &'a MessageManager,
}

pub struct RenderContext<'a> {
    pub editor: EditorRenderContext<'a>,
    pub input: InputRenderContext<'a>,
    pub config: &'a Config,
    pub diagnostics: DiagnosticRenderContext<'a>,
}
