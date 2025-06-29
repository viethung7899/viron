use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Drawable, RenderContext};

pub struct StatusLine {
    id: String,
    height: usize,
    offset_bottom: usize,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            id: "status_line".to_string(),
            height: 1,
            offset_bottom: 1,
        }
    }
}

impl Drawable for StatusLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, buffer: &mut RenderBuffer, context: &RenderContext) {
        let row = buffer.height - self.height - self.offset_bottom;

        let left = format!(" {} ", context.mode.to_name().to_uppercase());

        let cursor = context.cursor.get_position();
        let right = format!(" {}:{} ", cursor.row + 1, cursor.column + 1);

        let file = format!(
            " {}{}",
            context
                .document
                .file_name()
                .as_deref()
                .unwrap_or("new file"),
            if context.document.modified {
                " [+]"
            } else {
                ""
            }
        );
        let center_width = buffer.width - left.len() - right.len();
        let center = format!("{file:<center_width$}");

        let colors = match context.mode {
            Mode::Normal => context.theme.colors.status.normal,
            Mode::Insert => context.theme.colors.status.insert,
            _ => context.theme.colors.status.command,
        };

        let mut outer = Style::from(colors);
        outer.bold = true;
        let inner = Style::from(context.theme.colors.status.inner);

        buffer.set_text(row, 0, &left, &outer);
        buffer.set_text(row, left.len(), &center, &inner);
        buffer.set_text(row, left.len() + center_width, &right, &outer);
    }
}
