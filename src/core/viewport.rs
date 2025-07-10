use crate::core::buffer::Buffer;
use crate::core::cursor::Cursor;

/// Viewport manages which part of the buffer is visible on screen
#[derive(Debug)]
pub struct Viewport {
    start_row: usize,
    start_column: usize,
    width: usize,
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

    pub fn content_width(&self, gutter_width: usize) -> usize {
        self.width.saturating_sub(gutter_width)
    }

    /// Scrolls the viewport to ensure the cursor is visible, accounting for gutter
    pub fn scroll_to_cursor_with_gutter(&mut self, cursor: &Cursor, gutter_width: usize) -> bool {
        let (row, column) = cursor.get_display_cursor();
        let content_width = self.content_width(gutter_width);

        let mut scrolled = false;

        // Scroll vertically if needed (unchanged)
        if row < self.start_row {
            self.start_row = row;
            scrolled = true;
        } else if row >= self.start_row + self.height {
            self.start_row = row - self.height + 1;
            scrolled = true;
        }

        // Scroll horizontally if needed (accounting for reduced content width)
        if column < self.start_column {
            self.start_column = column;
            scrolled = true;
        } else if column >= self.start_column + content_width {
            self.start_column = column - content_width + 1;
            scrolled = true;
        }

        scrolled
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
