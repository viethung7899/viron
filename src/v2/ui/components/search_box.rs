use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, RenderContext};

pub struct SearchBox;

impl Drawable for SearchBox {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let search_buffer = context.search_buffer;

        match context.mode {
            Mode::Search => {
                let search = search_buffer.buffer.content();
                let formatted = format!("/{search:<width$}");
                buffer.set_text(start_row, 0, &formatted, &context.theme.editor_style());
            }
            _ => {
                let last_search = search_buffer.last_search.clone();
                if last_search.is_empty() || search_buffer.results.is_empty() {
                    self.clear(buffer, context)?;
                } else {
                    let formatted = format!("/{last_search:<width$}");
                    buffer.set_text(start_row, 0, &formatted, &context.theme.editor_style());
                }
            }
        }

        Ok(())
    }

    fn bounds(&self, buffer: &RenderBuffer, context: &RenderContext) -> Bounds {
        Bounds {
            start_row: buffer.height - 1,
            start_col: 0,
            width: buffer.width,
            height: 1,
        }
    }
}
