use crate::core::{buffer::Buffer, command::CommandBuffer};
use regex::Regex;
use tree_sitter::Point;

#[derive(Debug, Clone, Default)]
pub struct SearchBuffer {
    pub buffer: CommandBuffer,

    // Search results
    pub last_search: String,
    pub results: Vec<Point>,
    pub current: Option<usize>,
}

impl SearchBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.last_search.clear();
        self.results.clear();
        self.current = None;
    }

    pub fn search(&mut self, pattern: &str, buffer: &Buffer) -> anyhow::Result<()> {
        self.reset();
        self.last_search = pattern.to_string();
        let regex = Regex::new(pattern)?;

        // Find all matches in the buffer content
        self.results = buffer
            .to_string()
            .lines()
            .enumerate()
            .map(|(r, line)| {
                regex
                    .find_iter(line)
                    .filter_map(|m| byte_to_char_index(line, m.start()))
                    .map(|c| Point { row: r, column: c })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        log::info!("{:?}", self.results);

        Ok(())
    }

    pub fn find_first(&mut self, point: &Point) -> Option<Point> {
        if self.results.is_empty() {
            self.current = None;
            return None;
        }
        // Binary search for the first occurrence
        let index = match self.results.binary_search(point) {
            Ok(i) => i,
            Err(i) => i.checked_sub(1).unwrap_or(0),
        };
        self.current = Some(index);
        Some(self.results[index].clone())
    }

    pub fn find_next(&mut self, point: &Point) -> Option<Point> {
        if self.results.is_empty() {
            self.current = None;
            return None;
        }
        let len = self.results.len();
        // Binary search for the next occurrence
        let index = match self.results.binary_search(point) {
            Ok(i) => i + 1,
            Err(i) => i,
        } % len;
        self.current = Some(index);
        Some(self.results[index].clone())
    }

    pub fn find_previous(&mut self, point: &Point) -> Option<Point> {
        if self.results.is_empty() {
            self.current = None;
            return None;
        }
        let len = self.results.len();
        let index = match self.results.binary_search(point) {
            Ok(i) => i.checked_sub(1).unwrap_or(len - 1),
            Err(i) => i.checked_sub(1).unwrap_or(len - 1),
        };
        self.current = Some(index);
        Some(self.results[index].clone())
    }
}

fn byte_to_char_index(s: &str, byte_index: usize) -> Option<usize> {
    // Check if byte_index is on a character boundary
    if !s.is_char_boundary(byte_index) {
        return None;
    }

    // Count characters up to the byte index
    Some(s[..byte_index].chars().count())
}
