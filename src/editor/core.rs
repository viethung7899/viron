use crate::constants::RESERVED_ROW_COUNT;
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
}

