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
}

impl MessageManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_message(&self) -> Option<&Message> {
        self.current_message.as_ref()
    }

    pub fn show_message(&mut self, message: Message) {
        self.current_message = Some(message);
    }

    pub fn clear_message(&mut self) {
        self.current_message = None;
    }
}
