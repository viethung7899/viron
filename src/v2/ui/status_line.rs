use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
};
use std::io::Write;

use crate::core::document::Document;
use crate::editor::Mode;

pub struct StatusLine {
    height: usize,
    width: usize,
}

impl StatusLine {
    pub fn new(width: usize) -> Self {
        Self { height: 1, width }
    }

    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        document: &Document,
        mode: &Mode,
        cursor_pos: &tree_sitter::Point,
        screen_height: usize,
    ) -> Result<()> {
        // Position at the bottom of the screen
        let status_line_row = screen_height - self.height;
        queue!(writer, cursor::MoveTo(0, status_line_row as u16))?;

        // Build status line content
        let mode_str = format!("{:10}", mode.to_name());

        let file_name = document
            .file_name()
            .unwrap_or_else(|| "[No Name]".to_string());
        let modified_indicator = if document.modified { "[+]" } else { "" };

        let file_info = format!("{}{}", file_name, modified_indicator);

        let cursor_info = format!("{}:{}", cursor_pos.row + 1, cursor_pos.column + 1);

        // Calculate spacing to right-align cursor position
        let padding_length = self
            .width
            .saturating_sub(mode_str.len() + file_info.len() + cursor_info.len() + 2);
        let padding = " ".repeat(padding_length);

        let status = format!("{} {} {}{}", mode_str, file_info, padding, cursor_info);

        // Render with inverted colors
        let styled_status = status.stylize().with(Color::Black).on(Color::White);

        queue!(writer, PrintStyledContent(styled_status))?;

        Ok(())
    }
}
