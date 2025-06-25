use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{PrintStyledContent, Stylize},
    terminal,
};
use std::io::Write;

use crate::core::{buffer::Buffer, cursor::Cursor, viewport::Viewport};

pub struct Renderer<W: Write> {
    writer: W,
    screen_width: usize,
    screen_height: usize,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W) -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            writer,
            screen_width: width as usize,
            screen_height: height as usize,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        queue!(
            self.writer,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    pub fn render_buffer(&mut self, buffer: &mut Buffer, viewport: &Viewport) -> Result<()> {
        // Get visible content from viewport
        let top_line = viewport.top_line();

        for line_idx in 0..viewport.height() {
            let buffer_line = top_line + line_idx;
            if buffer_line >= buffer.line_count() {
                break;
            }

            let content = buffer.get_content_line(buffer_line);
            let offset = viewport.offset();
            let visible_content = if offset.column < content.len() {
                &content[offset.column..]
            } else {
                ""
            };

            // Move to line position and print content
            queue!(
                self.writer,
                cursor::MoveTo(0, line_idx as u16),
                PrintStyledContent(visible_content.stylize())
            )?;
        }

        Ok(())
    }

    pub fn position_cursor(&mut self, cursor: &Cursor, viewport: &Viewport) -> Result<()> {
        let cursor_pos = cursor.get_position();
        let screen_pos = (
            (cursor_pos.column - viewport.offset().column) as u16,
            (cursor_pos.row - viewport.top_line()) as u16,
        );

        queue!(self.writer, cursor::MoveTo(screen_pos.0, screen_pos.1))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.screen_width, self.screen_height)
    }
}
