use crate::constants::RESERVED_ROW_COUNT;
use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, RenderContext};
use anyhow::Ok;

pub struct StatusLine;

impl Drawable for StatusLine {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let document = &context.document;

        let left = format!(" {} ", context.mode.to_name().to_uppercase());

        let (row, column) = context.cursor.get_display_cursor();
        let right = format!(" {}:{} ", row + 1, column + 1);

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
            Mode::Command => context.theme.colors.status.command,
            Mode::Search => context.theme.colors.status.search,
        };

        let mut outer = Style::from(colors);
        outer.bold = true;
        let inner = Style::from(context.theme.colors.status.inner);

        buffer.set_text(start_row, 0, &left, &outer);
        buffer.set_text(start_row, left.len(), &center, &inner);
        buffer.set_text(start_row, left.len() + center_width, &right, &outer);

        Ok(())
    }

    fn bounds(&self, render_buffer: &RenderBuffer, _context: &RenderContext) -> Bounds {
        Bounds {
            start_row: render_buffer.height - RESERVED_ROW_COUNT,
            start_col: 0,
            width: render_buffer.width,
            height: 1,
        }
    }
}
