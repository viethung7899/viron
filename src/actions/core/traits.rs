use crate::config::Config;
use crate::core::buffer::Buffer;
use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::document::Document;
use crate::core::mode::Mode;
use crate::core::viewport::Viewport;
use crate::service::LspService;
use crate::service::lsp::LspClient;
use crate::ui::components::ComponentIds;

pub trait DocumentAccess<'a> {
    fn get_document(&'a self) -> &'a Document;
    fn get_document_mut(&'a mut self) -> &'a mut Document;
    fn get_buffer(&'a self) -> &'a Buffer;
    fn get_buffer_mut(&'a mut self) -> &'a mut Buffer;
    fn get_buffer_manager(&'a mut self) -> &'a mut BufferManager;
}

pub trait CursorAccess<'a> {
    fn cursor_mut(&'a mut self) -> &'a mut Cursor;
    fn viewport_mut(&'a mut self) -> &'a mut Viewport;
}

pub trait ModeAccess<'a> {
    fn current_mode(&'a self) -> &'a Mode;
    fn set_mode(&'a mut self, mode: Mode);
}

pub trait ConfigAccess<'a> {
    fn config(&'a self) -> &'a Config;
}

pub trait UIAccess<'a> {
    fn mark_dirty(&'a mut self, id: &'a str) -> anyhow::Result<()>;
    fn mark_all_dirty(&'a mut self);
    fn mark_visibility_dirty(&'a mut self, id: &'a str, visible: bool) -> anyhow::Result<()>;
    fn component_ids(&'a self) -> &'a ComponentIds;
}

pub trait LspAccess<'a> {
    fn lsp_client_mut(&mut self) -> Option<&'a mut LspClient>;
    fn lsp_service(&mut self) -> &'a mut LspService;
}

pub trait AppControl<'a> {
    fn quit(&'a mut self);
}