use crate::core::buffer::Buffer;
use crate::core::cursor::Cursor;

/// Viewport manages which part of the buffer is visible on screen
#[derive(Debug)]
pub struct Viewport {
    start_row: usize,
    start_column: usize,
    /// Number of columns visible in the viewport
    width: usize,
    /// Number of lines visible in the viewport
    height: usize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            start_row: 0,
            start_column: 0,
            width: 80,
            height: 24,
        }
    }
}

impl Viewport {
    pub fn new(width: usize, height: usize) -> Self {
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

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    /// Returns the index of the first visible line
    pub fn top_line(&self) -> usize {
        self.start_row
    }

    /// Returns the index of the first visible column
    pub fn left_column(&self) -> usize {
        self.start_column
    }

    /// Scrolls the viewport to ensure the cursor is visible
    /// Returns true if the viewport was scrolled
    pub fn scroll_to_cursor(&mut self, cursor: &Cursor) -> bool {
        let (row, column) = cursor.get_display_cursor();

        // Scroll vertically if needed
        if row < self.start_row {
            // Cursor is above viewport
            self.start_row = row;
            return true;
        } else if row >= self.start_row + self.height {
            // Cursor is below viewport
            self.start_row = row - self.height + 1;
            return true;
        }

        // Scroll horizontally if needed
        if column < self.start_column {
            // Cursor is to the left of viewport
            self.start_column = column;
            return true;
        } else if column >= self.start_column + self.width {
            // Cursor is to the right of viewport
            self.start_column = column - self.width + 1;
            return true;
        }

        false
    }

    /// Scrolls up by the specified number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.start_row = self.start_row.saturating_sub(lines);
    }

    /// Scrolls down by the specified number of lines
    pub fn scroll_down(&mut self, lines: usize, buffer: &Buffer) {
        let max_top = buffer.line_count().saturating_sub(self.height);
        self.start_row = (self.start_row + lines).min(max_top);
    }

    /// Scrolls left by the specified number of columns
    pub fn scroll_left(&mut self, columns: usize) {
        self.start_column = self.start_column.saturating_sub(columns);
    }

    /// Scrolls right by the specified number of columns
    pub fn scroll_right(&mut self, columns: usize) {
        self.start_column += columns;
    }

    /// Centers the viewport on a specific line
    pub fn center_on_line(&mut self, line: usize, buffer: &Buffer) {
        let half_height = self.height / 2;

        if line < half_height {
            self.start_row = 0;
        } else {
            let max_top = buffer.line_count().saturating_sub(self.height);
            self.start_row = (line - half_height).min(max_top);
        }
    }
}
