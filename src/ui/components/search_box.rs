use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, Focusable, RenderContext};

pub struct SearchBox;

impl Drawable for SearchBox {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()> {
        let Bounds {
            start_row, width, ..
        } = self.bounds(buffer, context);
        let search_buffer = context.search_buffer;
        let error_style = Style {
            foreground: context.theme.colors.diagnostic.error.foreground,
            background: context.theme.colors.editor.background,
            ..Default::default()
        };

        match context.mode {
            Mode::Search => {
                let search = search_buffer.buffer.content();
                let formatted = format!("/{search:<width$}");
                buffer.set_text(start_row, 0, &formatted, &context.theme.editor_style());
            }
            _ => {
                let last_search = search_buffer.last_search.clone();
                if last_search.is_empty() {
                    let message = format!("{:<width$}", "E: No search pattern");
                    buffer.set_text(start_row, 0, &message, &error_style);
                    return Ok(());
                };

                if let Some(index) = search_buffer.current {
                    let counter = format!("[{}/{}]", index + 1, search_buffer.results.len());
                    buffer.set_text(
                        start_row,
                        0,
                        &format!("{counter:>width$}"),
                        &context.theme.editor_style(),
                    );
                    buffer.set_text(
                        start_row,
                        0,
                        &format!("/{last_search}"),
                        &context.theme.editor_style(),
                    );
                } else {
                    let formatted =
                        format!("{:<width$}", format!("E: No pattern found: {last_search}"));
                    buffer.set_text(start_row, 0, &formatted, &error_style);
                }
            }
        }

        Ok(())
    }

    fn bounds(&self, buffer: &RenderBuffer, _context: &RenderContext) -> Bounds {
        Bounds {
            start_row: buffer.height - 1,
            start_col: 0,
            width: buffer.width - 10,
            height: 1,
        }
    }
}

impl Focusable for SearchBox {
    fn get_display_cursor(&self, buffer: &RenderBuffer, context: &RenderContext) -> (usize, usize) {
        let search_buffer = context.search_buffer;
        let cursor_col = if context.mode == &Mode::Search {
            search_buffer.buffer.cursor_position() + 1
        } else {
            search_buffer.last_search.len() + 1
        };
        (buffer.height - 1, cursor_col)
    }
}
