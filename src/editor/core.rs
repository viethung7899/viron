use crate::config::editor::Gutter;
use crate::constants::{MIN_GUTTER_WIDTH, RESERVED_ROW_COUNT};
use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::document::Document;
use crate::core::mode::Mode;
use crate::core::viewport::Viewport;

pub struct EditorCore {
    pub buffer_manager: BufferManager,
    pub cursor: Cursor,
    pub viewport: Viewport,
    pub mode: Mode,
}

impl EditorCore {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer_manager: BufferManager::new(),
            cursor: Cursor::new(),
            viewport: Viewport::new(width, height - RESERVED_ROW_COUNT),
            mode: Mode::Normal,
        }
    }

    pub fn current_document(&self) -> &Document {
        self.buffer_manager.current()
    }

    pub fn current_document_mut(&mut self) -> &mut Document {
        self.buffer_manager.current_mut()
    }

    pub fn resize_viewport(&mut self, width: usize, height: usize) {
        self.viewport.resize(width, height - RESERVED_ROW_COUNT);
    }

    pub fn scroll_viewport(&mut self, has_gutter: bool) -> bool {
        let line_count = self.current_document().buffer.line_count();
        let gutter_width = if has_gutter {
            0
        } else {
            (line_count.to_string().len() + 1).max(MIN_GUTTER_WIDTH)
        };
        self
            .viewport
            .scroll_to_cursor_with_gutter(&self.cursor, gutter_width)
    }
}

