use crate::theme::Style;

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

#[derive(Debug, Clone)]
pub(super) struct RenderBuffer {
    pub(super) cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl RenderBuffer {
    pub(super) fn new(width: usize, height: usize, default_style: Option<Style>) -> Self {
        let cells = vec![
            Cell {
                c: ' ',
                style: default_style.unwrap_or_default(),
            };
            width * height
        ];
        Self {
            cells,
            width,
            height,
        }
    }

    pub(super) fn set_cell(&mut self, x: usize, y: usize, c: char, style: &Style) {
        if let Some(current) = self.cells.get_mut(y * self.width + x) {
            *current = Cell {
                c,
                style: style.clone(),
            };
        }
    }

    pub(super) fn set_text(&mut self, x: usize, y: usize, text: &str, style: &Style) {
        let position = y * self.width + x;
        for (i, c) in text.chars().enumerate() {
            if let Some(current) = self.cells.get_mut(position + i) {
                *current = Cell {
                    c,
                    style: style.clone(),
                };
            }
        }
    }

    pub(super) fn diff(&self, other: &Self) -> Vec<Change> {
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
}
