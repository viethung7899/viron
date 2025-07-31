use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::input::InputProcessor;
use crate::input::events::EventHandler;

pub struct InputSystem {
    pub command_buffer: CommandBuffer,
    pub search_buffer: SearchBuffer,
    pub input_state: InputProcessor,
    pub event_handler: EventHandler,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            command_buffer: CommandBuffer::new(),
            search_buffer: SearchBuffer::new(),
            input_state: InputProcessor::new(),
            event_handler: EventHandler::new(),
        }
    }

    pub fn clear_all(&mut self) {
        self.command_buffer.clear();
        self.search_buffer.buffer.clear();
        self.input_state.clear();
    }

    pub fn is_input_pending(&self) -> bool {
        !self.input_state.is_empty()
    }
}
