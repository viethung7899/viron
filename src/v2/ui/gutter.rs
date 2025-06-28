use anyhow::Result;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Drawable, RenderContext};
use crate::ui::theme::Style;

pub struct Gutter {
    pub width: usize,
}

impl Gutter {
    pub fn new() -> Self {
        Self {
            width: 4,
        }
    }

    pub fn with_options(width: usize) -> Self {
        Self {
            width,
        }
    }
}

impl Drawable for Gutter {
    fn draw(&self, buffer: &mut RenderBuffer, context: &RenderContext) {
        let top_line = context.viewport.top_line();
        let style = Style::from(context.theme.colors.gutter);

        for i in 0..context.viewport.height() {
            let buffer_line = top_line + i;
            if buffer_line >= context.document.buffer.line_count() {
                break;
            }

            let line_number = buffer_line + 1; // 1-indexed line numbers
            let line_text = format!("{:width$}", line_number, width = self.width);

            buffer.set_text(i, 0, &line_text, &style);
        }
    }
}
