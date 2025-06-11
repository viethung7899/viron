use std::str::FromStr;

use tree_sitter::Point;

use crate::{buffer::gap_buffer::GapBuffer, editor};

mod gap_buffer;

pub struct Buffer {
    buffer: GapBuffer<u8>,
    line_starts: Vec<usize>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            buffer: GapBuffer::default(),
            line_starts: vec![0],
        }
    }
}

impl Buffer {
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn from_str(s: &str) -> Self {
        let slice = s.as_bytes();
        let mut lines_start = vec![0];
        for (i, &byte) in slice.iter().enumerate() {
            if byte == b'\n' {
                lines_start.push(i + 1);
            }
        }
        Self {
            buffer: GapBuffer::from_slice(&slice),
            line_starts: lines_start,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(&self.buffer.buffer[..self.buffer.gap_start]);
        result.extend(&self.buffer.buffer[self.buffer.gap_end..]);
        result
    }

    pub fn cursor_position(&self, cursor: &Point) -> usize {
        self.line_starts[cursor.row] + cursor.column
    }

    pub fn insert_char(&mut self, ch: char, cursor: &mut Point) {
        let position = self.cursor_position(cursor);
        let s = ch.to_string();
        let bytes = s.as_bytes();
        self.buffer.move_gap(position);
        self.buffer.insert_multiple(bytes);

        for line in self.line_starts[cursor.row + 1..].iter_mut() {
            *line += bytes.len();
        }

        if ch == '\n' {
            self.line_starts.insert(cursor.row + 1, position + 1);
            cursor.column = 0;
            cursor.row += 1;
        } else {
            cursor.column += bytes.len();
        }
    }

    pub fn move_left(&self, cursor: &mut Point) {
        cursor.column = cursor.column.saturating_sub(1);
    }

    pub fn move_right(&self, cursor: &mut Point, mode: &editor::Mode) {
        cursor.column += 1;
        self.clamp_column(cursor, mode);
    }

    pub fn move_up(&self, cursor: &mut Point, mode: &editor::Mode) {
        if cursor.row == 0 {
            return;
        }
        cursor.row -= 1;
        self.clamp_column(cursor, mode);
    }

    pub fn move_down(&self, cursor: &mut Point, mode: &editor::Mode) {
        if cursor.row == self.line_starts.len() - 1 {
            return;
        }
        cursor.row += 1;
        self.clamp_column(cursor, mode);
    }

    pub fn move_to_line_start(&self, cursor: &mut Point) {
        cursor.column = 0;
    }

    pub fn move_to_line_end(&self, cursor: &mut Point, mode: &editor::Mode) {
        cursor.column = usize::MAX;
        self.clamp_column(cursor, mode);
    }

    fn clamp_column(&self, cursor: &mut Point, mode: &editor::Mode) {
        let line_end = if cursor.row + 1 < self.line_starts.len() {
            self.line_starts[cursor.row + 1] - 1
        } else {
            self.buffer.len_without_gap()
        };
        let mut line_length = line_end - self.line_starts[cursor.row];
        if mode != &editor::Mode::Insert {
            line_length = line_length.saturating_sub(1);
        }
        cursor.column = cursor.column.min(line_length);
    }
}
