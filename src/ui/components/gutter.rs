use crate::config::editor::Gutter as GutterConfig;
use crate::constants::{MIN_GUTTER_WIDTH, RESERVED_ROW_COUNT};
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, RenderContext};
use anyhow::Result;

pub struct Gutter;

impl Gutter {
    pub fn get_width(&self, context: &RenderContext) -> usize {
        if context.config.gutter == GutterConfig::None {
            return 0;
        }
        let line_count = context.document.buffer.line_count();
        let digits = line_count.to_string().len();
        (digits + 1).max(MIN_GUTTER_WIDTH)
    }

    fn draw_absolute(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        let Bounds {
            start_col,
            width,
            height,
            ..
        } = self.bounds(buffer, context);
        let top_line = context.viewport.top_line();
        let line_count = context.document.buffer.line_count();
        let style = Style::from(context.config.theme.colors.gutter);

        for i in 0..(height) {
            let line = top_line + i;
            if line >= line_count {
                break;
            }
            let line_text = format!("{:>w$}", line + 1, w = width - 1);

            buffer.set_text(i, start_col, &line_text, &style);
        }

        Ok(())
    }

    fn draw_relative(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        let Bounds {
            start_col,
            width,
            height,
            ..
        } = self.bounds(buffer, context);
        let top_line = context.viewport.top_line();
        let line_count = context.document.buffer.line_count();
        let (current_line, _) = context.cursor.get_display_cursor();
        let style = Style::from(context.config.theme.colors.gutter);

        for i in 0..(height) {
            let line = top_line + i;
            if line >= line_count {
                break;
            }
            let distance = line.abs_diff(current_line);
            let line_text = if distance == 0 {
                format!("{:<w$}", line + 1, w = width - 1)
            } else {
                format!("{:>w$}", distance, w = width - 1)
            };

            buffer.set_text(i, start_col, &line_text, &style);
        }

        Ok(())
    }
}

impl Drawable for Gutter {
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        match context.config.gutter {
            GutterConfig::Absolute => self.draw_absolute(buffer, context),
            GutterConfig::Relative => self.draw_relative(buffer, context),
            _ => {
                return Ok(());
            }
        }
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
