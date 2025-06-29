use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, RenderContext};

pub struct Gutter {
    id: String,
}

impl Gutter {
    pub fn new() -> Self {
        Self {
            id: "gutter".to_string(),
        }
    }
}

impl Drawable for Gutter {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let top_line = context.viewport.top_line();
        let style = Style::from(context.theme.colors.gutter);
        let Bounds {
            start_col,
            width,
            height,
            ..
        } = self.bounds(buffer.get_size(), context);
        let line_count = context.buffer_manager.current_buffer().line_count();

        for i in 0..(height) {
            let buffer_line = top_line + i;
            if buffer_line >= line_count {
                break;
            }

            let line_number = buffer_line + 1; // 1-indexed line numbers
            let line_text = format!("{line_number:>w$}", w = width - 1);

            buffer.set_text(i, start_col, &line_text, &style);
        }

        Ok(())
    }

    fn bounds(&self, _size: (usize, usize), context: &RenderContext) -> Bounds {
        Bounds {
            start_row: 0,
            start_col: 0,
            width: context.gutter_width,
            height: context.viewport.height(),
        }
    }
}
