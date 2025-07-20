use crate::config::editor::Gutter as GutterConfig;
use crate::constants::{MIN_GUTTER_WIDTH, RESERVED_ROW_COUNT};
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable};
use anyhow::Result;
use crate::ui::context::RenderContext;

pub struct Gutter;

impl Gutter {
    pub fn get_width(&self, context: &RenderContext) -> usize {
        if context.config.gutter == GutterConfig::None {
            return 0;
        }
        let line_count = context.editor.document.buffer.line_count();
        let digits = line_count.to_string().len();
        (digits + 1).max(MIN_GUTTER_WIDTH)
    }

    fn get_line_text(&self, context: &RenderContext, current_line: usize, line: usize) -> String {
        match context.config.gutter {
            GutterConfig::None => String::new(),
            GutterConfig::Absolute => {
                format!("{:>w$}", line + 1, w = self.get_width(context) - 1)
            }
            GutterConfig::Relative => {
                let distance = line.abs_diff(current_line);
                if distance == 0 {
                    format!("{:<w$}", line + 1, w = self.get_width(context) - 1)
                } else {
                    format!("{:>w$}", distance, w = self.get_width(context) - 1)
                }
            }
        }
    }
}

impl Drawable for Gutter {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        if context.config.gutter == GutterConfig::None {
            return Ok(());
        }
        let Bounds {
            start_col,
            width,
            height,
            ..
        } = self.bounds(buffer, context);
        let top_line = context.editor.viewport.top_line();
        let line_count = context.editor.document.buffer.line_count();
        let style = Style::from(context.config.theme.colors.gutter);
        let (current_line, _) = context.editor.cursor.get_display_cursor();

        for i in 0..height {
            let line = top_line + i;
            let line_text = if line >= line_count {
                " ".repeat(width - 1)
            } else {
                self.get_line_text(context, current_line, line)
            };
            buffer.set_text(i, start_col, &line_text, &style);
        }

        Ok(())
    }

    fn bounds(&self, buffer: &RenderBuffer, context: &RenderContext) -> Bounds {
        Bounds {
            start_row: 0,
            start_col: 0,
            width: self.get_width(context),
            height: buffer.height - RESERVED_ROW_COUNT,
        }
    }
}
