use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
};
use std::io::Write;

use crate::core::buffer::Buffer;
use crate::core::viewport::Viewport;

pub struct Gutter {
    width: usize,
    show_line_numbers: bool,
}

impl Gutter {
    pub fn new() -> Self {
        Self {
            width: 4,
            show_line_numbers: true,
        }
    }

    pub fn with_options(width: usize, show_line_numbers: bool) -> Self {
        Self {
            width,
            show_line_numbers,
        }
    }

    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        buffer: &Buffer,
        viewport: &Viewport,
    ) -> Result<()> {
        if !self.show_line_numbers {
            return Ok(());
        }

        let top_line = viewport.top_line();

        for i in 0..viewport.height() {
            let buffer_line = top_line + i;
            if buffer_line >= buffer.line_count() {
                break;
            }

            let line_number = buffer_line + 1; // 1-indexed line numbers
            let line_text = format!("{:width$}", line_number, width = self.width);

            queue!(
                writer,
                cursor::MoveTo(0, i as u16),
                PrintStyledContent(line_text.stylize().with(Color::DarkGrey))
            )?;
        }

        Ok(())
    }

    pub fn width(&self) -> usize {
        if self.show_line_numbers {
            self.width
        } else {
            0
        }
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }
}
