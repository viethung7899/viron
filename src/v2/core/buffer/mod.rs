use log::info;
use tree_sitter::Point;

use crate::core::buffer::gap_buffer::GapBuffer;

pub mod gap_buffer;

#[derive(Debug)]
pub struct Buffer {
    buffer: GapBuffer<char>,
    line_starts: Vec<usize>,
    // pub diagnostics: Vec<Diagnostic>,
}

impl Default for Buffer {
    fn default() -> Self {
        let mut buffer = GapBuffer::default();
        buffer.insert_single('\n');
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
        let prefix = &self.buffer.buffer[..self.buffer.gap_start];
        let suffix = &self.buffer.buffer[self.buffer.gap_end..];
        prefix.iter().chain(suffix.iter()).collect()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn from_string(content: &str) -> Self {
        let chars = content.chars().collect::<Vec<_>>();
        let mut lines_start = vec![0];
        for (i, &char) in chars.iter().enumerate() {
            if char == '\n' {
                lines_start.push(i + 1);
            }
        }
        Self {
            buffer: GapBuffer::from_slice(&chars),
            line_starts: lines_start,
            ..Default::default()
        }
    }

    pub fn get_content_line(&mut self, line: usize) -> String {
        if line > self.line_count() {
            return "".to_string();
        }
        let line_start = self.cursor_position(&Point {
            row: line,
            column: 0,
        });
        self.buffer.move_gap(line_start);
        let line_end = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1]
        } else {
            self.buffer.len_without_gap()
        };
        let index_start = self.buffer.gap_end;
        let index_end = line_end + self.buffer.gap_end - line_start;
        let chars = &self.buffer.buffer[index_start..index_end];
        chars.iter().collect()
    }

    pub fn get_line_length(&self, line: usize) -> usize {
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
        // Move gap to insertion position
        self.buffer.move_gap(position);

        // Insert the character
        self.buffer.insert_single(ch);

        // Update line starts for all lines after the current one
        let row = self.row_at_position(position);
        for line in self.line_starts[row + 1..].iter_mut() {
            *line += 1;
        }

        // If inserting a newline, add a new line start
        if ch == '\n' {
            self.line_starts.insert(row + 1, position + 1);
        }

        // Return the position after insertion
        position + 1
    }

    pub fn insert_string(&mut self, position: usize, string: &str) {
        for (index, ch) in string.chars().enumerate() {
            self.insert_char(position + index, ch);
        }
    }

    pub fn delete_char(&mut self, position: usize) -> Option<(char, usize)> {
        if position >= self.buffer.len_without_gap() {
            return None;
        }

        self.buffer.move_gap(position);

        let Some(deleted_char) = self.buffer.delete_single() else {
            return None;
        };

        // Update line starts for all lines after the current one
        let row = self.row_at_position(position);

        if deleted_char == '\n' {
            if row + 1 < self.line_starts.len() {
                self.line_starts.remove(row + 1);
            }
        }

        for line in self.line_starts[row + 1..].iter_mut() {
            *line -= 1;
        }

        Some((deleted_char, position))
    }

    // pub fn move_left(&self, cursor: &mut Point, mode: &Mode) {
    //     if cursor.column > 0 {
    //         cursor.column -= 1;
    //         return;
    //     }
    //     if cursor.row > 0 {
    //         cursor.row -= 1;
    //         cursor.column = usize::MAX;
    //         self.clamp_column(cursor, mode);
    //     }
    // }

    // pub fn move_right(&self, cursor: &mut Point, mode: &Mode) {
    //     let previous = cursor.column;
    //     cursor.column += 1;
    //     self.clamp_column(cursor, mode);
    //     if previous == cursor.column && cursor.row + 1 < self.line_count() {
    //         cursor.row += 1;
    //         cursor.column = 0;
    //     }
    // }

    // pub fn move_up(&self, cursor: &mut Point, mode: &Mode) {
    //     if cursor.row == 0 {
    //         return;
    //     }
    //     cursor.row -= 1;
    //     self.clamp_column(cursor, mode);
    // }

    // pub fn move_down(&self, cursor: &mut Point, mode: &Mode) {
    //     if cursor.row == self.line_starts.len() - 1 {
    //         return;
    //     }
    //     cursor.row += 1;
    //     self.clamp_column(cursor, mode);
    // }

    // pub fn move_to_line_start(&self, cursor: &mut Point) {
    //     cursor.column = 0;
    // }

    // pub fn move_to_line_end(&self, cursor: &mut Point, mode: &Mode) {
    //     cursor.column = usize::MAX;
    //     self.clamp_column(cursor, mode);
    // }

    // pub fn get_current_char(&self, cursor: &Point) -> Option<char> {
    //     let position = self.cursor_position(cursor);
    //     if position < self.buffer.gap_start {
    //         Some(self.buffer.buffer[position])
    //     } else {
    //         let index = position + self.buffer.gap_len();
    //         self.buffer.buffer.get(index).copied()
    //     }
    // }

    // pub fn delete_current_line(&mut self, cursor: &mut Point) -> Option<String> {
    //     cursor.column = 0;
    //     let position = self.cursor_position(cursor);
    //     self.buffer.move_gap(position);
    //     let line_end = if cursor.row + 1 < self.line_starts.len() {
    //         self.line_starts[cursor.row + 1]
    //     } else {
    //         self.buffer.len_without_gap()
    //     };
    //     let line_length = line_end - self.line_starts[cursor.row];
    //     let Some(chars) = self.buffer.delete(line_length) else {
    //         return None;
    //     };
    //     for line in self.line_starts[cursor.row + 1..].iter_mut() {
    //         *line -= line_length;
    //     }
    //     let lines = self.line_count();
    //     if lines > 1 {
    //         self.line_starts.remove(cursor.row);
    //         cursor.row = cursor.row.min(self.line_count() - 1);
    //     } else {
    //         self.line_starts[0] = 0;
    //         cursor.row = 0;
    //     }
    //     self.dirty = true;
    //     Some(chars.into_iter().collect())
    // }

    // pub fn find_next_word(&mut self, cursor: &Point) -> Option<Point> {
    //     self.buffer.move_gap(self.cursor_position(cursor));
    //     let mut index = self.buffer.gap_end;
    //     let mut point = cursor.clone();
    //     let buffer = &self.buffer.buffer;
    //     let length = buffer.len();
    //     let mode = Mode::Insert;

    //     // Skip the current word
    //     if index < length && !buffer[index].is_whitespace() {
    //         let keyword_type = is_keyword(buffer[index]);

    //         while index < length {
    //             let c = buffer[index];
    //             if c.is_whitespace() || is_keyword(c) != keyword_type {
    //                 break;
    //             }
    //             index += 1;
    //             self.move_right(&mut point, &mode);
    //         }
    //     }

    //     // Skip the whitespace
    //     let mut already_new_line = false;
    //     while index < length && buffer[index].is_whitespace() {
    //         let c = buffer[index];
    //         if c == '\n' && already_new_line {
    //             return Some(point);
    //         }
    //         already_new_line = c == '\n';
    //         index += 1;
    //         self.move_right(&mut point, &mode);
    //     }

    //     Some(point)
    // }

    // pub fn delete_word_inline(&mut self, cursor: &mut Point) -> String {
    //     let position = self.cursor_position(cursor);
    //     self.buffer.move_gap(position);
    //     let mut index = self.buffer.gap_end;
    //     let buffer = &self.buffer.buffer;
    //     let length = buffer.len();

    //     // Delete the current word
    //     if index < length && !buffer[index].is_whitespace() {
    //         let keyword_type = is_keyword(buffer[index]);

    //         while index < length {
    //             let c = buffer[index];
    //             if c.is_whitespace() || is_keyword(c) != keyword_type {
    //                 break;
    //             }
    //             index += 1;
    //         }
    //     }

    //     // Delete the whitespace
    //     while index < length && buffer[index].is_whitespace() && buffer[index] != '\n' {
    //         index += 1;
    //     }

    //     let deleted = String::from_iter(&buffer[self.buffer.gap_end..index]);
    //     let mut count = deleted.len();

    //     while count > 0 {
    //         self.delete_char(cursor, &Mode::Normal);
    //         count -= 1;
    //     }

    //     deleted
    // }

    // pub fn find_previous_word(&mut self, cursor: &Point) -> Option<Point> {
    //     self.buffer.move_gap(self.cursor_position(cursor));
    //     let mut point = cursor.clone();
    //     let buffer = &self.buffer.buffer;
    //     let mode = Mode::Insert;

    //     let mut index = self.buffer.gap_start.saturating_sub(1);
    //     self.move_left(&mut point, &mode);

    //     while index > 0 && buffer[index].is_whitespace() {
    //         if buffer[index] == '\n' && buffer[index - 1] == '\n' {
    //             return Some(point);
    //         }
    //         index -= 1;
    //         self.move_left(&mut point, &mode);
    //     }

    //     let keyword_type = is_keyword(buffer[index]);
    //     while index > 0 {
    //         let prev = buffer[index - 1];
    //         if prev.is_whitespace() || is_keyword(buffer[index - 1]) != keyword_type {
    //             return Some(point);
    //         }
    //         index -= 1;
    //         point.column -= 1;
    //     }

    //     if index == 0 {
    //         return Some(Point { row: 0, column: 0 });
    //     } else {
    //         return Some(point);
    //     }
    // }

    // pub fn clamp_column(&self, cursor: &mut Point, mode: &Mode) {
    //     let line_end = if cursor.row + 1 < self.line_starts.len() {
    //         self.line_starts[cursor.row + 1] - 1
    //     } else {
    //         self.buffer.len_without_gap()
    //     };
    //     let mut line_length = line_end - self.line_starts[cursor.row];
    //     if mode != &Mode::Insert {
    //         line_length = line_length.saturating_sub(1);
    //     }
    //     cursor.column = cursor.column.min(line_length);
    // }

    // pub fn save(&mut self) -> Result<String> {
    //     let file = &self.file.as_ref().context("No file name")?;
    //     let bytes = self.to_bytes();
    //     std::fs::write(file, &bytes)?;
    //     let message = format!(
    //         "{:?}, {}L, {}B written",
    //         file,
    //         self.line_count(),
    //         bytes.len()
    //     );
    //     self.dirty = false;
    //     Ok(message)
    // }

    // pub fn uri(&self) -> Result<Option<String>> {
    //     match &self.file {
    //         Some(file) => Ok(format!(
    //             "file://{}",
    //             utils::absolutize(file)?.to_string_lossy().to_string()
    //         )
    //         .into()),
    //         None => Ok(None),
    //     }
    // }

    /// Helper method to determine which row a position is in
    fn row_at_position(&self, position: usize) -> usize {
        // Find the row by binary search (more efficient for large files)
        match self.line_starts.binary_search(&position) {
            Ok(row) => row,
            Err(row) => row - 1,
        }
    }

    /// Convert position to a Point
    pub fn point_at_position(&self, position: usize) -> Point {
        let row = self.row_at_position(position);
        let column = position - self.line_starts[row];
        Point { row, column }
    }

    // pub fn offer_diagnostics(&mut self, message: &TextDocumentPublishDiagnostics) -> Result<()> {
    //     let Some(uri) = self.uri()? else {
    //         return Ok(());
    //     };

    //     if let Some(diagnostics_uri) = &message.uri {
    //         if &uri != diagnostics_uri {
    //             return Ok(());
    //         }
    //     }

    //     self.diagnostics.extend(
    //         message
    //             .diagnostics
    //             .iter()
    //             .filter(|d| d.is_for(&uri))
    //             .map(|d| d.clone()),
    //     );

    //     Ok(())
    // }

    // pub fn diagnostics_for_lines(
    //     &self,
    //     starting_line: usize,
    //     ending_line: usize,
    // ) -> Vec<&Diagnostic> {
    //     self.diagnostics
    //         .iter()
    //         .filter(|d| {
    //             let start = &d.range.start;
    //             start.line >= starting_line && start.line < ending_line
    //         })
    //         .collect::<Vec<_>>()
    // }
}

fn is_keyword(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
