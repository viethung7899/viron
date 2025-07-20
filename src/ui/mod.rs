use crate::ui::render_buffer::RenderBuffer;
use context::RenderContext;

pub(crate) mod components;
pub mod compositor;
pub mod render_buffer;
pub mod theme;
pub mod context;

pub struct Bounds {
    pub start_row: usize,
    pub start_col: usize,
    pub width: usize,
    pub height: usize,
}

pub trait Drawable {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()>;
    fn bounds(&self, buffer: &RenderBuffer, context: &RenderContext) -> Bounds;

    fn clear(&self, buffer: &mut RenderBuffer, context: &RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row,
            start_col,
            width,
            height,
        } = self.bounds(buffer, context);

        for row in start_row..start_row + height {
            buffer.set_text(
                row,
                start_col,
                &" ".repeat(width),
                &context.config.theme.editor_style(),
            );
        }

        Ok(())
    }
}

pub trait Focusable {
    fn get_display_cursor(&self, buffer: &RenderBuffer, context: &RenderContext) -> (usize, usize);
}
