use super::theme::Style;
use anyhow::Result;
use crossterm::{cursor, style, QueueableCommand};
use std::fmt::{Debug, Write as DebugWrite};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

#[derive(Debug, Clone)]
pub struct Change<'a> {
    pub x: usize,
    pub y: usize,
    pub cell: &'a Cell,
}

impl<'a> Change<'a> {
    pub(super) fn flush<W: Write>(&self, writer: &mut W, style: &Style) -> Result<()> {
        let style = self.cell.style.to_content_style(&style);
        let content = style::StyledContent::new(style, self.cell.c);
        writer
            .queue(cursor::MoveTo(self.x as u16, self.y as u16))?
            .queue(style::Print(content))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct RenderBuffer {
    pub(super) cells: Vec<Cell>,
    pub(super) width: usize,
    pub(super) height: usize,
}

impl Debug for RenderBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RenderBuffer\n")?;
        for i in 0..self.height {
            let start = i * self.width;
            let end = start + self.width;
            for cell in &self.cells[start..end] {
                let format = if cell.c == ' ' { '·' } else { cell.c };
                f.write_char(format)?;
            }
            f.write_str("\n")?
        }
        Ok(())
    }
}

impl RenderBuffer {
    pub(super) fn new(width: usize, height: usize) -> Self {
        let cells = vec![
            Cell {
                c: ' ',
                style: Style::default(),
            };
            width * height
        ];
        Self {
            cells,
            width,
            height,
        }
    }

    pub(super) fn set_cell(&mut self, row: usize, col: usize, c: char, style: &Style) {
        if col >= self.width || row >= self.height {
            return;
        }
        if let Some(current) = self.cells.get_mut(row * self.width + col) {
            *current = Cell {
                c,
                style: style.clone(),
            };
        }
    }

    pub(super) fn set_text(&mut self, row: usize, col: usize, text: &str, style: &Style) {
        if row >= self.height {
            return;
        }
        let position = row * self.width + col;
        for (index, c) in text.chars().enumerate() {
            if index + col >= self.width {
                break;
            }
            if let Some(current) = self.cells.get_mut(position + index) {
                *current = Cell {
                    c,
                    style: style.clone(),
                };
            }
        }
    }

    pub fn diff(&self, other: &Self) -> Vec<Change> {
        let mut changes = Vec::new();
        for (pos, cell) in self.cells.iter().enumerate() {
            if *cell != other.cells[pos] {
                let x = pos % self.width;
                let y = pos / self.width;
                changes.push(Change { x, y, cell });
            }
        }
        changes
    }

    pub(super) fn flush<W: Write>(&self, writer: &mut W, editor_style: &Style) -> Result<()> {
        writer.queue(cursor::MoveTo(0, 0))?;
        for cell in self.cells.iter() {
            let style = cell.style.to_content_style(editor_style);
            let content = style::StyledContent::new(style, cell.c);
            writer.queue(style::Print(content))?;
        }
        Ok(())
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}
