use tree_sitter::Point;

use crate::{
    buffer::gap_buffer::GapBuffer,
    editor::{self, Mode},
    log,
};

pub mod gap_buffer;

pub struct Buffer {
    buffer: GapBuffer<char>,
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
        let chars = s.chars().collect::<Vec<_>>();
        let mut lines_start = vec![0];
        for (i, &char) in chars.iter().enumerate() {
            if char == '\n' {
                lines_start.push(i + 1);
            }
        }
        Self {
            buffer: GapBuffer::from_slice(&chars),
            line_starts: lines_start,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let prefix = &self.buffer.buffer[..self.buffer.gap_start];
        let suffix = &self.buffer.buffer[self.buffer.gap_end..];
        let content: String = prefix.iter().chain(suffix.iter()).collect();
        content.into_bytes()
    }

    pub fn cursor_position(&self, cursor: &Point) -> usize {
        self.line_starts[cursor.row] + cursor.column
    }

    pub fn insert_char(&mut self, ch: char, cursor: &mut Point) {
        let position = self.cursor_position(cursor);
        self.buffer.move_gap(position);
        self.buffer.insert_single(ch);

        for line in self.line_starts[cursor.row + 1..].iter_mut() {
            *line += 1;
        }

        if ch == '\n' {
            self.line_starts.insert(cursor.row + 1, position + 1);
            cursor.column = 0;
            cursor.row += 1;
        } else {
            cursor.column += 1;
        }
    }

    pub fn insert_string(&mut self, string: &str, cursor: &mut Point) {
        for ch in string.chars() {
            self.insert_char(ch, cursor);
        }
    }

    pub fn move_left(&self, cursor: &mut Point, mode: &editor::Mode) {
        if cursor.column > 0 {
            cursor.column -= 1;
            return;
        }
        if cursor.row > 0 {
            cursor.row -= 1;
            cursor.column = usize::MAX;
            self.clamp_column(cursor, mode);
        }
    }

    pub fn move_right(&self, cursor: &mut Point, mode: &editor::Mode) {
        let previous = cursor.column;
        cursor.column += 1;
        self.clamp_column(cursor, mode);
        if previous == cursor.column && cursor.row + 1 < self.line_count() {
            cursor.row += 1;
            cursor.column = 0;
        }
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

    pub fn delete_char(&mut self, cursor: &mut Point, mode: &editor::Mode) -> Option<char> {
        let position = self.cursor_position(cursor);
        self.buffer.move_gap(position);

        let Some(char) = self.buffer.delete_single() else {
            return None;
        };

        for line in self.line_starts[cursor.row + 1..].iter_mut() {
            *line -= 1;
        }

        if char == '\n' {
            self.line_starts.remove(cursor.row + 1);
            cursor.row = cursor.row.saturating_sub(1);
        }
        self.clamp_column(cursor, mode);
        Some(char)
    }

    pub fn get_current_char(&self, cursor: &Point) -> Option<char> {
        let position = self.cursor_position(cursor);
        if position < self.buffer.gap_start {
            Some(self.buffer.buffer[position])
        } else {
            let index = position + self.buffer.gap_len();
            self.buffer.buffer.get(index).copied()
        }
    }

    pub fn delete_current_line(&mut self, cursor: &mut Point) -> Option<String> {
        cursor.column = 0;
        let position = self.cursor_position(cursor);
        self.buffer.move_gap(position);
        let line_end = if cursor.row + 1 < self.line_starts.len() {
            self.line_starts[cursor.row + 1]
        } else {
            self.buffer.len_without_gap()
        };
        let line_length = line_end - self.line_starts[cursor.row];
        let Some(chars) = self.buffer.delete(line_length) else {
            return None;
        };
        for line in self.line_starts[cursor.row + 1..].iter_mut() {
            *line -= line_length;
        }
        let lines = self.line_count();
        if lines > 1 {
            self.line_starts.remove(cursor.row);
            cursor.row = cursor.row.min(self.line_count() - 1);
        } else {
            self.line_starts[0] = 0;
            cursor.row = 0;
        }
        Some(chars.into_iter().collect())
    }

    pub fn find_next_word(&mut self, cursor: &Point) -> Option<Point> {
        self.buffer.move_gap(self.cursor_position(cursor));
        let mut index = self.buffer.gap_end;
        let mut point = cursor.clone();
        let buffer = &self.buffer.buffer;
        let length = buffer.len();
        let mode = Mode::Insert;

        log!("[next word] before {point:?}");

        // Skip the current word
        if index < length && !buffer[index].is_whitespace() {
            let keyword_type = is_in_keyword(buffer[index]);

            while index < length {
                let c = buffer[index];
                if c.is_whitespace() || is_in_keyword(c) != keyword_type {
                    break;
                }
                index += 1;
                self.move_right(&mut point, &mode);
            }
        }

        log!("[next word] first pass {point:?}");

        // Skip the whitespace
        let mut already_new_line = false;
        while index < length && buffer[index].is_whitespace() {
            let c = buffer[index];
            if c == '\n' && already_new_line {
                log!("[next word] second new line");
                return Some(point);
            }
            already_new_line = c == '\n';
            index += 1;
            self.move_right(&mut point, &mode);
        }

        Some(point)
    }

    pub fn find_previous_word(&mut self, cursor: &Point) -> Option<Point> {
        self.buffer.move_gap(self.cursor_position(cursor));
        let mut point = cursor.clone();
        let buffer = &self.buffer.buffer;
        let mode = Mode::Insert;

        let mut index = self.buffer.gap_start.saturating_sub(1);
        self.move_left(&mut point, &mode);

        while index > 0 && buffer[index].is_whitespace() {
            if buffer[index] == '\n' && buffer[index - 1] == '\n' {
                return Some(point);
            }
            index -= 1;
            self.move_left(&mut point, &mode);
        }

        let keyword_type = is_in_keyword(buffer[index]);
        while index > 0 {
            let prev = buffer[index - 1];
            if prev.is_whitespace() || is_in_keyword(buffer[index - 1]) != keyword_type {
                return Some(point);
            }
            index -= 1;
            point.column -= 1;
        }

        if index == 0 {
            return Some(Point { row: 0, column: 0 });
        } else {
            return Some(point);
        }
    }

    pub fn clamp_column(&self, cursor: &mut Point, mode: &editor::Mode) {
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

fn is_in_keyword(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
