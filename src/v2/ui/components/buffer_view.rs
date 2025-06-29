use crate::core::buffer;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, RenderContext};
use anyhow::{Ok, Result};

pub struct BufferView {
    id: String,
}

impl BufferView {
    pub fn new() -> Self {
        Self {
            id: "buffer_view".to_string(),
        }
    }

    fn render_plain_text(
        &self,
        render_buffer: &mut RenderBuffer,
        context: &mut RenderContext,
    ) -> Result<()> {
        let Bounds {
            start_col,
            width: visible_width,
            height: visible_height,
            ..
        } = self.bounds(render_buffer.get_size(), context);
        let viewport = context.viewport;
        let buffer = context.buffer_manager.current_buffer_mut();
        let theme = context.theme;

        let top_line = viewport.top_line();
        let left_col = viewport.left_column();
        let editor_style = theme.editor_style();

        for viewport_row in 0..visible_height {
            let buffer_row = top_line + viewport_row;

            let content = if buffer_row >= buffer.line_count() {
                " ".repeat(visible_width)
            } else {
                format!(
                    "{:<visible_width$}",
                    buffer
                        .get_content_line(buffer_row)
                        .get(left_col..)
                        .unwrap_or("")
                )
            };

            render_buffer.set_text(viewport_row, start_col, &content, &editor_style);
        }

        Ok(())
    }

    fn render_with_syntax_highlighting(
        &self,
        buffer: &mut RenderBuffer,
        context: &RenderContext,
    ) -> Result<()> {
        Ok(())
    }
}

impl Drawable for BufferView {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        self.render_plain_text(buffer, context)
    }

    fn bounds(&self, size: (usize, usize), context: &RenderContext<'_>) -> Bounds {
        let width = context.viewport.width();
        let height = context.viewport.height();
        let window_width = size.0;
        Bounds {
            start_row: 0,
            start_col: window_width - width,
            width,
            height,
        }
    }
}
