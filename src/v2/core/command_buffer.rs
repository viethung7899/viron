#[derive(Debug, Clone, Default)]
pub struct CommandBuffer {
    content: Vec<char>,
    cursor_position: usize,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn content(&self) -> String {
        self.content.iter().collect()
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.empty() {
            return false;
        }
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
            self.cursor_position = self
                .cursor_position
                .min(self.content.len().saturating_sub(1));
        }
        return true;
    }

    pub fn backspace(&mut self) -> bool {
        if self.empty() {
            return false;
        }
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
        true
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }
}
