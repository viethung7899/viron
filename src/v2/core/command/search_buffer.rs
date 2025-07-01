use crate::core::{buffer::Buffer, command::CommandBuffer};
use regex::Regex;
use tree_sitter::Point;

#[derive(Debug, Clone, Default)]
pub struct SearchBuffer {
    pub buffer: CommandBuffer,

    // Search results
    pub last_search: String,
    pub results: Vec<Point>,
}

impl SearchBuffer {
    pub fn new() -> Self {
        Self {
            buffer: CommandBuffer::new(),
            last_search: String::new(),
            results: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.last_search.clear();
        self.results.clear();
    }

    pub fn search(&mut self, buffer: &Buffer) -> anyhow::Result<()> {
        let content = self.buffer.content();
        let regex = Regex::new(&content)?;
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
        self.last_search = content;
        Ok(())
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
