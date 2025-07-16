use crate::actions::core::traits::{AppControl, ConfigAccess, CursorAccess, DocumentAccess, ModeAccess, UIAccess};
use crate::config::Config;
use crate::core::buffer::Buffer;
use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::document::Document;
use crate::core::mode::Mode;
use crate::core::viewport::Viewport;
use crate::editor::Editor;
use crate::ui::components::ComponentIds;

impl<'a> DocumentAccess<'a> for Editor {
    fn get_document(&'a self) -> &'a Document {
        self.buffer_manager.current()
    }

    fn get_document_mut(&'a mut self) -> &'a mut Document {
        self.buffer_manager.current_mut()
    }

    fn get_buffer(&'a self) -> &'a Buffer {
        self.buffer_manager.current_buffer()
    }

    fn get_buffer_mut(&'a mut self) -> &'a mut Buffer {
        self.buffer_manager.current_buffer_mut()
    }

    fn get_buffer_manager(&'a mut self) -> &'a mut BufferManager {
        &mut self.buffer_manager
    }
}

impl<'a> CursorAccess<'a> for Editor {
    fn cursor_mut(&'a mut self) -> &'a mut Cursor {
        &mut self.cursor
    }

    fn viewport_mut(&'a mut self) -> &'a mut Viewport {
        &mut self.viewport
    }
}

impl<'a> ModeAccess<'a> for Editor {
    fn current_mode(&'a self) -> &'a Mode {
        &self.mode
    }
    fn set_mode(&'a mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl<'a> ConfigAccess<'a> for Editor {
    fn config(&'a self) -> &'a Config {
        &self.config
    }
}

impl<'a> UIAccess<'a> for Editor {
    fn mark_dirty(&'a mut self, id: &'a str) -> anyhow::Result<()> {
        self.compositor.mark_dirty(id)
    }

    fn mark_all_dirty(&'a mut self) {
        self.compositor.mark_all_dirty();
    }

    fn mark_visibility_dirty(&'a mut self, id: &'a str, visible: bool) -> anyhow::Result<()> {
        self.compositor.mark_visible(id, visible)
    }

    fn component_ids(&'a self) -> &'a ComponentIds {
        &self.component_ids
    }
}

impl<'a> AppControl<'a> for Editor {
    fn quit(&'a mut self) {
        self.running = false;
    }
}