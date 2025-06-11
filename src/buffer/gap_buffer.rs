use tree_sitter::Point;

#[derive(Debug)]
struct LineInfo {
    start: usize,
    length: usize,
}

fn parse_lines(bytes: &[u8]) -> Vec<LineInfo> {
    let mut lines = Vec::new();
    let mut start = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            lines.push(LineInfo {
                start,
                length: i - start,
            });
            start = i + 1;
        }
    }

    // Add final line (or empty line if ends with \n)
    lines.push(LineInfo {
        start,
        length: bytes.len() - start,
    });

    lines
}

#[derive(Debug)]
pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
    lines: Vec<LineInfo>,
}

const INITIAL_CAPACITY: usize = 1024;

impl Default for GapBuffer {
    fn default() -> Self {
        Self {
            buffer: vec![0; INITIAL_CAPACITY],
            gap_start: 0,
            gap_end: INITIAL_CAPACITY,
            lines: vec![LineInfo {
                start: 0,
                length: 0,
            }],
        }
    }
}

impl GapBuffer {
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let length = bytes.len();
        let gap_length = length.max(INITIAL_CAPACITY);
        let capacity = INITIAL_CAPACITY + gap_length;

        let mut buffer = Vec::with_capacity(capacity);
        buffer.extend_from_slice(bytes);
        buffer.resize(capacity, 0);

        let lines = parse_lines(bytes);

        Self {
            buffer,
            gap_start: length,
            gap_end: capacity,
            lines,
        }
    }

    pub fn insert_char(&mut self, c: char, cursor: &mut Point) {
        self.move_gap_to_cursor(cursor);
        let mut encoded = [0; 4];
        let bytes = c.encode_utf8(&mut encoded).as_bytes();

        if self.gap_len() < bytes.len() {
            self.expand_gap();
        }

        // Update line structure for newlines
        if c == '\n' {
            self.split_line(cursor.row);
        }

        for (i, &byte) in bytes.iter().enumerate() {
            self.buffer[self.gap_start + i] = byte;
        }

        if c == '\n' {
            cursor.row += 1;
            cursor.column = 0;
        } else {
            cursor.column += 1;
        }

        self.gap_start += bytes.len();
    }

    pub fn delete(&mut self, pos: usize) {
        if pos < self.gap_start {
            self.gap_start -= 1;
        } else {
            self.gap_end += 1;
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn split_line(&mut self, row: usize) {
        let new_line = LineInfo {
            start: self.gap_start + 1, // After newline character
            length: 0,
        };
        self.lines.insert(row + 1, new_line);
    }

    fn move_gap_to_cursor(&mut self, cursor: &Point) {
        let target_pos = self.get_buffer_position(cursor);
        self.move_gap(target_pos);
    }

    fn get_buffer_position(&self, cursor: &Point) -> usize {
        let line_info = &self.lines[cursor.row];
        line_info.start + cursor.column.min(line_info.length)
    }

    fn move_gap(&mut self, new_pos: usize) {
        if new_pos < self.gap_start {
            let distance = self.gap_start - new_pos;
            self.buffer
                .copy_within(new_pos..self.gap_start, self.gap_end - distance);
            self.gap_start = new_pos;
            self.gap_end -= distance;
        } else if new_pos > self.gap_start {
            let distance = new_pos - self.gap_start;
            self.buffer
                .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    fn gap_len(&self) -> usize {
        self.gap_end - self.gap_start
    }

    fn expand_gap(&mut self) {
        let new_size = self.buffer.len() * 2;
        let mut new_buffer = vec![0; new_size];

        let prefix_len = self.gap_start;
        let suffix_len = self.buffer.len() - self.gap_end;

        new_buffer[..prefix_len].copy_from_slice(&self.buffer[..prefix_len]);
        new_buffer[new_size - suffix_len..].copy_from_slice(&self.buffer[self.gap_end..]);

        self.gap_end = new_size - suffix_len;
        self.buffer = new_buffer;
    }

    pub fn move_left(&mut self, cursor: &mut Point) {
        if cursor.column > 0 {
            cursor.column -= 1;
        } else if cursor.row > 0 {
            cursor.row -= 1;
            cursor.column = self.lines[cursor.row].length;
        }
    }

    pub fn move_right(&mut self, cursor: &mut Point) {
        if cursor.column < self.lines[cursor.row].length {
            cursor.column += 1;
        } else if cursor.row < self.lines.len() - 1 {
            cursor.row += 1;
            cursor.column = 0;
        }
    }

    pub fn move_vertical(&mut self, cursor: &mut Point, delta: i32) {
        let new_row = cursor.row as i32 + delta;
        if new_row >= 0 && new_row < self.lines.len() as i32 {
            cursor.row = new_row as usize;
            cursor.column = cursor.column.min(self.lines[cursor.row].length);
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.buffer.len() - self.gap_len());
        bytes.extend_from_slice(&self.buffer[..self.gap_start]);
        bytes.extend_from_slice(&self.buffer[self.gap_end..]);
        bytes
    }
}

impl ToString for GapBuffer {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.to_bytes()).to_string()
    }
}
