use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, Focusable};
use crate::ui::context::RenderContext;

pub struct CommandLine;

impl Drawable for CommandLine {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let command = context.input.command_buffer.content();
        let formatted = format!(":{command:<width$}");
        buffer.set_text(start_row, 0, &formatted, &context.config.theme.editor_style());
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

impl Focusable for CommandLine {
    fn get_display_cursor(&self, buffer: &RenderBuffer, context: &RenderContext) -> (usize, usize) {
        let command = context.input.command_buffer;
        let cursor_col = command.cursor_position() + 1;
        (buffer.height - 1, cursor_col)
    }
}
