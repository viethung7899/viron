use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, RenderContext};

pub struct CommandLine;

impl Drawable for CommandLine {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let command = context.command_buffer.content();
        let formatted = format!(":{command:<width$}");
        buffer.set_text(start_row, 0, &formatted, &context.theme.editor_style());
        Ok(())
    }

    fn bounds(&self, render_buffer: &RenderBuffer, _context: &RenderContext) -> Bounds {
        Bounds {
            start_row: render_buffer.height - 1,
            start_col: 0,
            width: render_buffer.width,
            height: 1,
        }
    }
}
