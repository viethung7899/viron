use tree_sitter::Point;

use crate::core::utf8::Utf8CharIterator;
use crate::core::{
    buffer::gap_buffer::GapBuffer,
    history::edit::{Delete, Edit, Insert},
};

pub mod gap_buffer;

#[derive(Debug)]
pub struct Buffer {
    buffer: GapBuffer<u8>,
    line_starts: Vec<usize>,
    // pub diagnostics: Vec<Diagnostic>,
}

impl Default for Buffer {
    fn default() -> Self {
        let mut buffer = GapBuffer::default();
        buffer.insert_single(b'\n');
        Self {
            buffer,
            line_starts: vec![0],
        }
    }
}

impl Buffer {
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn to_string(&self) -> String {
        let bytes = self.to_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let prefix = &self.buffer.buffer[..self.buffer.gap_start];
        let suffix = &self.buffer.buffer[self.buffer.gap_end..];
        prefix.iter().chain(suffix.iter()).copied().collect()
    }

    pub fn from_string(content: &str) -> Self {
        let chars = content.as_bytes();
        let mut lines_start = vec![0];
        for (i, &byte) in chars.iter().enumerate() {
            if byte == b'\n' {
                lines_start.push(i + 1);
            }
        }
        Self {
            buffer: GapBuffer::from_slice(&chars),
            line_starts: lines_start,
            ..Default::default()
        }
    }

    pub fn get_content_line_as_bytes(&self, line: usize) -> Vec<u8> {
        if line >= self.line_count() {
            return Vec::new();
        }
        let line_start = self.line_starts[line];
        let line_end = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1]
        } else {
            self.buffer.len_without_gap()
        };
        self.buffer
            .get_range(line_start..line_end)
            .copied()
            .collect()
    }

    pub fn get_content_line(&self, line: usize) -> String {
        let bytes = self.get_content_line_as_bytes(line);
        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn get_line_length(&self, line: usize) -> usize {
        if line >= self.line_count() {
            return 0;
        }

        let line_content = self.get_content_line(line);
        line_content.chars().count()
    }

    pub fn get_line_length_bytes(&self, line: usize) -> usize {
        if line > self.line_count() {
            return 0;
        }
        let line_end = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1] - 1
        } else {
            self.buffer.len_without_gap()
        };
        line_end - self.line_starts[line]
    }

    pub fn cursor_position(&self, cursor: &Point) -> usize {
        self.line_starts[cursor.row] + cursor.column
    }

    pub fn insert_char(&mut self, position: usize, ch: char) -> usize {
        let mut utf8_bytes = [0; 4];
        let utf8_str = ch.encode_utf8(&mut utf8_bytes);
        self.insert_bytes(position, utf8_str.as_bytes())
    }

    pub fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> usize {
        // Move gap to insertion byte_position
        self.buffer.move_gap(position);

        // Insert the character
        self.buffer.insert_multiple(bytes);

        // Update line starts for all lines after the current one
        let row = self.row_at_position(position);
        for line in self.line_starts[row + 1..].iter_mut() {
            *line += bytes.len();
        }

        // If inserting a newline, add a new line start
        let mut newlines = Vec::new();
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                let new_line_start = position + i + 1;
                newlines.push(new_line_start);
            }
        }

        if !newlines.is_empty() {
            let insert_row = self.row_at_position(position) + 1;

            // Insert all newlines at once using splice for better performance
            self.line_starts.splice(insert_row..insert_row, newlines);
        }

        // Return the byte_position after insertion
        position + bytes.len()
    }

    pub fn insert_string(&mut self, position: usize, string: &str) -> usize {
        // let mut position = position;
        // for ch in string.chars() {
        //     position = self.insert_char(position, ch);
        // }
        // position
        self.insert_bytes(position, string.as_bytes())
    }

    pub fn delete_char(&mut self, position: usize) -> Option<(char, usize)> {
        if position >= self.buffer.len_without_gap() {
            return None;
        }

        self.buffer.move_gap(position);

        // For UTF-8, we need to determine how many bytes to delete
        let remaining_bytes = self.buffer.len_without_gap() - position;
        if remaining_bytes == 0 {
            return None;
        }

        let first_byte = *self.buffer.get_range(position..position + 1).next()?;
        let char_len = if first_byte < 0x80 {
            1
        } else if first_byte < 0xE0 {
            2
        } else if first_byte < 0xF0 {
            3
        } else {
            4
        };

        let char_len = char_len.min(remaining_bytes);
        let mut char_bytes = Vec::with_capacity(char_len);

        for _ in 0..char_len {
            if let Some(byte) = self.buffer.delete_single() {
                char_bytes.push(byte);
            } else {
                break;
            }
        }

        let deleted_char = String::from_utf8_lossy(&char_bytes)
            .chars()
            .next()
            .unwrap_or('\0');

        // Update line starts for all lines after the current one
        let row = self.row_at_position(position);

        if char_bytes.contains(&b'\n') {
            if row + 1 < self.line_starts.len() {
                self.line_starts.remove(row + 1);
            }
        }

        for line in self.line_starts[row + 1..].iter_mut() {
            *line -= char_len;
        }

        Some((deleted_char, position))
    }

    pub fn delete_string(&mut self, position: usize, byte_count: usize) -> Option<(String, usize)> {
        if position >= self.buffer.len_without_gap()
            || position + byte_count > self.buffer.len_without_gap()
            || byte_count == 0
        {
            return None;
        }

        let bytes = self
            .buffer
            .get_range(position..position + byte_count)
            .cloned()
            .collect::<Vec<_>>();

        let char_count = Utf8CharIterator::new(bytes.as_slice()).count();
        let mut deleted_string = String::new();

        for _ in 0..char_count {
            if let Some((ch, _)) = self.delete_char(position) {
                deleted_string.push(ch);
            } else {
                break;
            }
        }

        Some((deleted_string, position))
    }

    pub fn apply_edit(&mut self, change: &Edit) {
        match change {
            Edit::Insert(Insert {
                start_byte: position,
                text,
                ..
            }) => {
                self.insert_string(*position, &text);
            }
            Edit::Delete(Delete {
                start_byte: position,
                text,
                ..
            }) => {
                for _ in text.chars() {
                    self.delete_char(*position);
                }
            }
            Edit::Multiple { edits: changes, .. } => {
                for change in changes {
                    self.apply_edit(change);
                }
            }
        }
    }

    /// Helper method to determine which row a byte_position is in
    fn row_at_position(&self, position: usize) -> usize {
        // Find the row by binary search (more efficient for large files)
        self.line_starts
            .binary_search(&position)
            .unwrap_or_else(|row| row - 1)
    }

    /// Convert byte_position to a Point
    pub fn point_at_position(&self, position: usize) -> Point {
        let row = self.row_at_position(position);
        let column = position - self.line_starts[row];
        Point { row, column }
    }
}
