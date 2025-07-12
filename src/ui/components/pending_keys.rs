use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, RenderContext};

const WIDTH: usize = 10;
pub struct PendingKeys;

impl Drawable for PendingKeys {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row,
            start_col,
            width,
            ..
        } = self.bounds(buffer, context);

        let pending_keys = context.pending_keys.to_string();
        if pending_keys.is_empty() {
            return self.clear(buffer, context);
        }
        let text = format!("  {:w$}", pending_keys, w = width - 2);

        buffer.set_text(
            start_row,
            start_col,
            &text,
            &context.config.theme.editor_style(),
        );

        Ok(())
    }

    fn bounds(&self, buffer: &RenderBuffer, _context: &RenderContext) -> Bounds {
        Bounds {
            start_row: buffer.height - 1,
            start_col: buffer.width - WIDTH,
            width: WIDTH,
            height: 1,
        }
    }
}
