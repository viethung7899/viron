use crate::{
    core::{buffer::Buffer, utf8::Utf8CharIterator},
    editor::Mode,
};
use tree_sitter::Point;

#[derive(Debug, Clone, Default)]
pub struct Cursor {
    row: usize,
    char_column: usize,
    preferred_column: usize,
    byte_column: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_display_cursor(&self) -> (usize, usize) {
        (self.row, self.char_column)
    }

    pub fn get_point(&self) -> Point {
        Point {
            row: self.row,
            column: self.byte_column,
        }
    }

    pub fn set_point(&mut self, position: Point, buffer: &Buffer) {
        self.row = position.row;
        self.byte_column = position.column;
        self.char_column = self.byte_to_char_column(buffer);
        self.preferred_column = self.char_column;
    }

    fn byte_to_char_column(&self, buffer: &Buffer) -> usize {
        let line_bytes = buffer.get_content_line_as_bytes(self.row);

        if self.byte_column >= line_bytes.len() {
            return Utf8CharIterator::new(&line_bytes).count();
        }

        let prefix = &line_bytes[..self.byte_column];
        return Utf8CharIterator::new(&prefix).count();
    }

    fn char_to_byte_column(&self, buffer: &Buffer) -> usize {
        let line_bytes = buffer.get_content_line_as_bytes(self.row);
        let mut iter = Utf8CharIterator::new(&line_bytes)
            .skip(self.char_column)
            .peekable();
        iter.peek().map(|item| item.byte_index).unwrap_or_default()
    }

    fn sync_byte_column(&mut self, buffer: &Buffer) {
        self.byte_column = self.char_to_byte_column(buffer);
    }

    /// Move cursor one character to the left
    pub fn move_left(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.char_column > 0 {
            self.char_column -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.char_column = buffer.get_line_length(self.row).saturating_sub(1);
            if *mode != Mode::Insert {
                // In non-insert mode, don't allow cursor to go beyond the last character
                self.char_column = self.char_column.saturating_sub(1);
            }
        }
        self.sync_byte_column(buffer);
        self.preferred_column = self.char_column;
    }

    /// Move cursor one character to the right
    pub fn move_right(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.row).saturating_sub(1);

        let at_end_of_line = if *mode == Mode::Insert {
            self.char_column >= line_length
        } else {
            self.char_column >= line_length.saturating_sub(1)
        };

        if !at_end_of_line {
            self.char_column += 1;
        } else if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.char_column = 0;
        }
        self.sync_byte_column(buffer);
        self.preferred_column = self.char_column;
    }

    /// Move cursor up one line
    pub fn move_up(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.row == 0 {
            return;
        }

        self.row -= 1;
        self.clamp_column(buffer, mode);
    }

    /// Move cursor down one line
    pub fn move_down(&mut self, buffer: &Buffer, mode: &Mode) {
        if self.row + 1 >= buffer.line_count() {
            return;
        }

        self.row += 1;
        self.clamp_column(buffer, mode);
    }

    /// Move to the start of the current line
    pub fn move_to_line_start(&mut self) {
        self.char_column = 0;
        self.preferred_column = 0;
    }

    /// Move to the end of the current line
    pub fn move_to_line_end(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.row).saturating_sub(1);
        if *mode == Mode::Insert {
            self.char_column = line_length;
        } else {
            self.char_column = line_length.saturating_sub(1);
        }
        self.sync_byte_column(buffer);
        self.preferred_column = self.char_column;
    }

    /// Jump to the next word
    pub fn find_next_word(&mut self, buffer: &Buffer) -> Cursor {
        // Get the position within the buffer
        let current_point = self.get_point();
        let position = buffer.cursor_position(&current_point);

        // Get buffer content
        let content = buffer.to_string();
        let chars: Vec<char> = content.chars().collect();

        if position >= chars.len() {
            return self.clone();
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
            let new_point = buffer.point_at_position(index);
            let mut new_cursor = Cursor {
                row: new_point.row,
                byte_column: new_point.column,
                char_column: 0,      // Will be calculated
                preferred_column: 0, // Will be set
            };
            new_cursor.char_column = new_cursor.byte_to_char_column(buffer);
            new_cursor.preferred_column = new_cursor.char_column;
            new_cursor
        } else {
            self.clone()
        }
    }

    /// Jump to the previous word
    pub fn find_previous_word(&self, buffer: &Buffer) -> Cursor {
        // Get the position within the buffer
        let current_point = self.get_point();
        let position = buffer.cursor_position(&current_point);

        if position == 0 {
            return self.clone();
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
            return Cursor {
                row: 0,
                char_column: 0,
                byte_column: 0,
                preferred_column: 0,
            };
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

        // Create new cursor at the target position
        let new_point = buffer.point_at_position(word_start);
        let mut new_cursor = Cursor {
            row: new_point.row,
            byte_column: new_point.column,
            char_column: 0,      // Will be calculated
            preferred_column: 0, // Will be set
        };
        new_cursor.char_column = new_cursor.byte_to_char_column(buffer);
        new_cursor.preferred_column = new_cursor.char_column;
        new_cursor
    }

    /// Ensure the cursor is at a valid position in the current line
    pub fn clamp_column(&mut self, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.row).saturating_sub(1);

        // In insert mode, cursor can be at the end of line
        // In normal mode, cursor can only be on the last character
        let max_column = if *mode == Mode::Insert {
            line_length
        } else {
            line_length.saturating_sub(1)
        };

        // Try to maintain the preferred column if possible
        self.char_column = self.preferred_column.min(max_column);
        self.sync_byte_column(buffer);
    }

    pub fn go_to_line(&mut self, line_number: usize, buffer: &Buffer, mode: &Mode) {
        let max_lines = buffer.line_count().saturating_sub(1);
        if line_number > max_lines {
            self.row = max_lines;
        } else {
            self.row = line_number;
        }
        self.clamp_column(buffer, mode);
    }

    pub fn go_to_column(&mut self, column: usize, buffer: &Buffer, mode: &Mode) {
        let line_length = buffer.get_line_length(self.row);
        if *mode == Mode::Insert {
            self.char_column = column.min(line_length);
        } else {
            self.char_column = column.min(line_length.saturating_sub(1));
        }
        self.sync_byte_column(buffer);
        self.preferred_column = self.char_column;
    }
}

fn is_keyword(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
