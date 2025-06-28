use super::theme::Style;

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
    pub(super) fn new(width: usize, height: usize) -> Self {
        let cells = vec![
            Cell {
                c: ' ',
                style: Style::default()
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
