use crate::{core::buffer::Buffer, editor::Mode};
use tree_sitter::Point;

#[derive(Debug, Default)]
pub struct Cursor {
    position: Point,
    // Store the preferred column for vertical movement
    preferred_column: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_position(&self) -> Point {
        self.position.clone()
    }

    pub fn set_position(&mut self, position: Point) {
        self.position = position;
        self.preferred_column = position.column;
    }

    /// Move cursor one character to the left
    pub fn move_left(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.position.column > 0 {
            self.position.column -= 1;
        } else if self.position.row > 0 {
            self.position.row -= 1;
            self.position.column = buffer.get_line_length(self.position.row);
            if *mode != Mode::Insert {
                // In non-insert mode, don't allow cursor to go beyond the last character
                self.position.column = self.position.column.saturating_sub(1);
            }
        }
    }

    /// Move cursor one character to the right
    pub fn move_right(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.position.row);

        let at_end_of_line = if *mode == Mode::Insert {
            self.position.column >= line_length
        } else {
            self.position.column >= line_length.saturating_sub(1)
        };

        if !at_end_of_line {
            self.position.column += 1;
        } else if self.position.row + 1 < buffer.line_count() {
            self.position.row += 1;
            self.position.column = 0;
        }

        self.preferred_column = self.position.column;
    }

    /// Move cursor up one line
    pub fn move_up(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.position.row == 0 {
            return;
        }

        self.position.row -= 1;
        self.clamp_column(buffer, mode);
    }

    /// Move cursor down one line
    pub fn move_down(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.position.row + 1 >= buffer.line_count() {
            return;
        }

        self.position.row += 1;
        self.clamp_column(buffer, mode);
    }

    /// Move to the start of the current line
    pub fn move_to_line_start(&mut self) {
        self.position.column = 0;
        self.preferred_column = 0;
    }

    /// Move to the end of the current line
    pub fn move_to_line_end(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.position.row);
        if *mode == Mode::Insert {
            self.position.column = line_length;
        } else {
            self.position.column = line_length.saturating_sub(1);
        }
        self.preferred_column = self.position.column;
    }
    
    /// Move to the start of the document
    pub fn move_to_top(&mut self) {
        self.position = Point { row: 0, column: 0 };
        self.preferred_column = 0;
    }
    
    /// Move to the end of the document
    pub fn move_to_bottom(&mut self, buffer: &Buffer, mode: &Mode) {
        let last_row = buffer.line_count().saturating_sub(1);
        let last_column = buffer.get_line_length(last_row).saturating_sub(1);
        self.position.row = last_row;
        self.position.column = if *mode == Mode::Insert {
            last_column
        } else {
            last_column.saturating_sub(1)
        };
        self.preferred_column = self.position.column;
    }

    /// Jump to the next word
    pub fn find_next_word(&mut self, buffer: &Buffer) {
        // Get the position within the buffer
        let position = buffer.cursor_position(&self.position);

        // Get buffer content
        let content = buffer.to_string();
        let chars: Vec<char> = content.chars().collect();

        if position >= chars.len() {
            return;
        }

        let mut index = position;

        // Skip the current word
        if !chars[index].is_whitespace() {
            let keyword_type = is_keyword(chars[index]);

            while index < chars.len()
                && !chars[index].is_whitespace()
                && is_keyword(chars[index]) == keyword_type
            {
                index += 1;
            }
        }

        // Skip whitespace
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        // Update the cursor position
        if index < chars.len() {
            self.position = buffer.point_at_position(index);
            self.preferred_column = self.position.column;
        }
    }

    /// Jump to the previous word
    pub fn find_previous_word(&mut self, buffer: &Buffer) {
        // Get the position within the buffer
        let position = buffer.cursor_position(&self.position);

        if position == 0 {
            return;
        }

        // Get buffer content
        let content = buffer.to_string();
        let chars: Vec<char> = content.chars().collect();

        let mut index = position.saturating_sub(1);

        // Skip whitespace backwards
        while index > 0 && chars[index].is_whitespace() {
            index -= 1;
        }

        if index == 0 {
            self.position = Point { row: 0, column: 0 };
            self.preferred_column = 0;
            return;
        }

        // Find the start of the current word
        let keyword_type = is_keyword(chars[index]);
        let mut word_start = index;

        while word_start > 0
            && !chars[word_start - 1].is_whitespace()
            && is_keyword(chars[word_start - 1]) == keyword_type
        {
            word_start -= 1;
        }

        self.position = buffer.point_at_position(word_start);
        self.preferred_column = self.position.column;
    }

    /// Ensure the cursor is at a valid position in the current line
    pub fn clamp_column(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.position.row);

        // In insert mode, cursor can be at the end of line
        // In normal mode, cursor can only be on the last character
        let max_column = if *mode == Mode::Insert {
            line_length
        } else {
            line_length.saturating_sub(1)
        };

        // Try to maintain the preferred column if possible
        self.position.column = self.preferred_column.min(max_column);
    }
    
    pub fn go_to_line(&mut self, line_number: usize, buffer: &Buffer, mode: &Mode) {
        let max_lines = buffer.line_count().saturating_sub(1);
        if line_number > max_lines {
            self.position.row = max_lines;
        } else {
            self.position.row = line_number;
        }
        self.clamp_column(buffer, mode);
    }
}

fn is_keyword(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
