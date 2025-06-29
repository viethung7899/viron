use anyhow::Ok;

use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, RenderContext};

pub struct StatusLine {
    id: String,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            id: "status_line".to_string(),
        }
    }
}

impl Drawable for StatusLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer.get_size(), context);
        let document = context.buffer_manager.current();

        let left = format!(" {} ", context.mode.to_name().to_uppercase());

        let cursor = context.cursor.get_position();
        let right = format!(" {}:{} ", cursor.row + 1, cursor.column + 1);

        let file = format!(
            " {}{}",
            document.file_name().as_deref().unwrap_or("new file"),
            if document.modified { " [+]" } else { "" }
        );
        let center_width = width - left.len() - right.len();
        let center = format!("{file:<center_width$}");

        let colors = match context.mode {
            Mode::Normal => context.theme.colors.status.normal,
            Mode::Insert => context.theme.colors.status.insert,
            _ => context.theme.colors.status.command,
        };

        let mut outer = Style::from(colors);
        outer.bold = true;
        let inner = Style::from(context.theme.colors.status.inner);

        buffer.set_text(start_row, 0, &left, &outer);
        buffer.set_text(start_row, left.len(), &center, &inner);
        buffer.set_text(start_row, left.len() + center_width, &right, &outer);

        Ok(())
    }

    fn bounds(&self, size: (usize, usize), _context: &RenderContext) -> Bounds {
        let (width, height) = size;
        let start_row = height - 2;
        Bounds {
            start_row,
            start_col: 0,
            width,
            height: 1,
        }
    }
}
