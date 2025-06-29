use crate::core::buffer::Buffer;
use crate::core::cursor::Cursor;
use tree_sitter::Point;

/// Viewport manages which part of the buffer is visible on screen
#[derive(Debug)]
pub struct Viewport {
    /// Starting position in the buffer (0-indexed for row and column)
    offset: Point,
    /// Number of columns visible in the viewport
    width: usize,
    /// Number of lines visible in the viewport
    height: usize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: Point { row: 0, column: 0 },
            width: 80,
            height: 24,
        }
    }
}

impl Viewport {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn offset(&self) -> Point {
        self.offset.clone()
    }

    pub fn resize(&mut self, height: usize, width: usize) {
        self.height = height;
        self.width = width;
    }

    /// Returns the index of the first visible line
    pub fn top_line(&self) -> usize {
        self.offset.row
    }

    /// Returns the index of the first visible column
    pub fn left_column(&self) -> usize {
        self.offset.column
    }

    /// Scrolls the viewport to ensure the cursor is visible
    /// Returns true if the viewport was scrolled
    pub fn scroll_to_cursor(&mut self, cursor: &Cursor) -> bool {
        let position = cursor.get_position();

        // Scroll vertically if needed
        if position.row < self.offset.row {
            // Cursor is above viewport
            self.offset.row = position.row;
            return true;
        } else if position.row >= self.offset.row + self.height {
            // Cursor is below viewport
            self.offset.row = position.row - self.height + 1;
            return true;
        }

        // Scroll horizontally if needed
        if position.column < self.offset.column {
            // Cursor is to the left of viewport
            self.offset.column = position.column;
            return true;
        } else if position.column >= self.offset.column + self.width {
            // Cursor is to the right of viewport
            self.offset.column = position.column - self.width + 1;
            return true;
        }

        false
    }

    /// Scrolls up by the specified number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.offset.row = self.offset.row.saturating_sub(lines);
    }

    /// Scrolls down by the specified number of lines
    pub fn scroll_down(&mut self, lines: usize, buffer: &Buffer) {
        let max_top = buffer.line_count().saturating_sub(self.height);
        self.offset.row = (self.offset.row + lines).min(max_top);
    }

    /// Scrolls left by the specified number of columns
    pub fn scroll_left(&mut self, columns: usize) {
        self.offset.column = self.offset.column.saturating_sub(columns);
    }

    /// Scrolls right by the specified number of columns
    pub fn scroll_right(&mut self, columns: usize) {
        self.offset.column += columns;
    }

    /// Centers the viewport on a specific line
    pub fn center_on_line(&mut self, line: usize, buffer: &Buffer) {
        let half_height = self.height / 2;

        if line < half_height {
            self.offset.row = 0;
        } else {
            let max_top = buffer.line_count().saturating_sub(self.height);
            self.offset.row = (line - half_height).min(max_top);
        }
    }

    /// Returns visible content lines from the buffer
    pub fn get_visible_lines<'a>(&self, buffer: &'a mut Buffer) -> Vec<String> {
        let mut lines = Vec::new();
        let end_line = (self.offset.row + self.height).min(buffer.line_count());

        for line_idx in self.offset.row..end_line {
            let line = buffer.get_content_line(line_idx);

            // Apply horizontal scrolling
            if self.offset.column < line.len() {
                let visible = &line[self.offset.column.min(line.len())..];
                lines.push(visible.to_string());
            } else {
                lines.push(String::new());
            }
        }

        lines
    }

    /// Translates buffer position to viewport coordinates
    pub fn buffer_to_viewport(&self, pos: &Point) -> Option<(usize, usize)> {
        if pos.row < self.offset.row || pos.row >= self.offset.row + self.height {
            return None;
        }

        if pos.column < self.offset.column || pos.column >= self.offset.column + self.width {
            return None;
        }

        Some((pos.row - self.offset.row, pos.column - self.offset.column))
    }

    /// Translates viewport coordinates to buffer position
    pub fn viewport_to_buffer(&self, row: usize, col: usize) -> Point {
        Point {
            row: self.offset.row + row,
            column: self.offset.column + col,
        }
    }

    /// Check if the given buffer position is visible in the viewport
    pub fn is_position_visible(&self, pos: &Point) -> bool {
        pos.row >= self.offset.row
            && pos.row < self.offset.row + self.height
            && pos.column >= self.offset.column
            && pos.column < self.offset.column + self.width
    }

    /// Clamps the viewport to ensure it doesn't show beyond buffer bounds
    pub fn clamp_to_buffer(&mut self, buffer: &Buffer) {
        // Don't scroll beyond the end of the buffer
        let max_top = buffer.line_count().saturating_sub(self.height);
        self.offset.row = self.offset.row.min(max_top);

        // We don't clamp offset.column here because lines can have different lengths
        // This is handled when rendering in get_visible_lines
    }
}
