use anyhow::Result;
use crossterm::{cursor, queue, style, terminal, QueueableCommand};
use std::io::Write;

use crate::core::{cursor::Cursor, viewport::Viewport};
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::{Style, Theme};

pub struct Renderer<W: Write> {
    writer: W,
    buffer: RenderBuffer,
    style: Style,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W, width: usize, height: usize, theme: &Theme) -> Result<Self> {
        let style = Style::from(theme.colors.editor);
        let buffer = RenderBuffer::new(width, height, &style);
        Ok(Self {
            writer,
            buffer,
            style
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

    // Output the render buffer
    pub fn render(&mut self) -> Result<()> {
        self.writer.queue(cursor::MoveTo(0, 0))?;
        for cell in self.buffer.cells.iter() {
            let style = cell.style.to_content_style(&self.style);
            let content = style::StyledContent::new(style, cell.c);
            self.writer.queue(style::Print(content))?;
        }
        Ok(())
    }

    // Compare the updated buffer with the old buffer,
    // and only updated cells
    pub fn render_diff(&mut self, updated: RenderBuffer) -> Result<()> {
        
        self.buffer = updated;
        Ok(())
    }

    pub fn position_cursor(&mut self, cursor: &Cursor, viewport: &Viewport) -> Result<()> {
        let cursor_pos = cursor.get_position();
        let screen_pos = (
            (cursor_pos.column - viewport.offset().column) as u16,
            (cursor_pos.row - viewport.top_line()) as u16,
        );

        self.writer.queue(cursor::MoveTo(screen_pos.0, screen_pos.1))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    pub fn writer(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn width(&self) -> usize {
        self.buffer.width
    }

    pub fn height(&self) -> usize {
        self.buffer.height
    }
}
