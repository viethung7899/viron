use std::fmt::{Debug, Write as DebugWrite};
use super::theme::{Style};
use anyhow::Result;
use crossterm::{QueueableCommand, cursor, style};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Cell {
    pub(super) c: char,
    pub(super) style: Style,
}

#[derive(Debug, Clone)]
pub(super) struct Change<'a> {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) cell: &'a Cell,
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
    pub style: Style,
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
    pub(super) fn new(width: usize, height: usize, fallback: &Style) -> Self {
        let cells = vec![
            Cell {
                c: ' ',
                style: fallback.clone(),
            };
            width * height
        ];
        Self {
            cells,
            style: fallback.clone(),
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

    pub(super) fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.c = ' ';
        }
    }

    pub(super) fn flush<W: Write>(&self, writer: &mut W) -> Result<()> {
        for (pos, cell) in self.cells.iter().enumerate() {
            let x = pos % self.width;
            let y = pos / self.width;
            let style = cell.style.to_content_style(&self.style);
            let content = style::StyledContent::new(style, cell.c);
            writer.queue(cursor::MoveTo(x as u16, y as u16))?.queue(style::Print(content))?;
        }
        Ok(())
    }
}
