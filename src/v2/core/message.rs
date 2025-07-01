#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub message_type: MessageType,
}

impl Message {
    pub fn info(content: String) -> Self {
        Self {
            content,
            message_type: MessageType::Info,
        }
    }

    pub fn error(content: String) -> Self {
        Self {
            content,
            message_type: MessageType::Error,
        }
    }
}

#[derive(Debug, Default)]
pub struct MessageManager {
    current_message: Option<Message>,
    clear_after_render: bool,
}

impl MessageManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_message(&mut self, message: Message) {
        self.current_message = Some(message);
        self.clear_after_render = true;
    }

    pub fn show_info(&mut self, content: String) {
        self.show_message(Message::info(content));
    }

    pub fn show_error(&mut self, content: String) {
        self.show_message(Message::error(content));
    }

    pub fn current_message(&self) -> Option<&Message> {
        self.current_message.as_ref()
    }

    pub fn has_message(&self) -> bool {
        self.current_message.is_some()
    }

    pub fn clear_message(&mut self) {
        self.current_message = None;
        self.clear_after_render = false;
    }

    /// Call this after rendering to clear the message
    pub fn post_render_cleanup(&mut self) {
        if self.clear_after_render {
            self.current_message = None;
            self.clear_after_render = false;
        }
    }

    /// Call this to keep the message for another render cycle
    pub fn keep_message(&mut self) {
        self.clear_after_render = false;
    }
}
