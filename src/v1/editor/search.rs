use regex::Regex;
use tree_sitter::Point;

use crate::v1::buffer::Buffer;

#[derive(Default)]
pub struct SearchBox {
    pub term: Option<Regex>,
    results: Vec<Point>,
}

impl SearchBox {
    pub fn clear(&mut self) {
        self.term = None;
        self.results.clear();
    }

    pub fn search(&mut self, term: &str, buffer: &Buffer) -> anyhow::Result<()> {
        let regex = Regex::new(term)?;
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
        self.term = Some(regex);
        Ok(())
    }

    pub fn find_next(&self, cursor: &Point) -> Option<(usize, Point)> {
        self.results
            .iter()
            .enumerate()
            .skip_while(|(_, position)| position <= &cursor)
            .next()
            .or(self.results.first().map(|first| (0, first)))
            .map(|(index, position)| (index, position.clone()))
    }

    pub fn find_previous(&self, cursor: &Point) -> Option<(usize, Point)> {
        self.results
            .iter()
            .enumerate()
            .take_while(|(_, position)| position < &cursor)
            .last()
            .or(self
                .results
                .last()
                .map(|last| (self.results.len().saturating_sub(1), last)))
            .map(|(index, position)| (index, position.clone()))
    }

    pub fn count(&self) -> usize {
        self.results.len()
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
