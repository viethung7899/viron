use crate::constants::{MIN_GUTTER_WIDTH, RESERVED_ROW_COUNT};
use crate::service::lsp::types::DiagnosticSeverity;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Style;
use crate::ui::{Bounds, Drawable, Focusable, RenderContext};
use anyhow::Result;
use std::ops::Add;
use std::str::from_utf8;
use tree_sitter::Point;

const DIAGNOSTIC_MARGIN: usize = 4;

pub struct EditorView;

impl EditorView {
    fn get_gutter_width(&self, context: &RenderContext<'_>) -> usize {
        let line_count = context.document.buffer.line_count();
        let digits = line_count.to_string().len();
        (digits + 1).max(MIN_GUTTER_WIDTH)
    }

    fn get_gutter_bounds(
        &self,
        render_buffer: &RenderBuffer,
        context: &RenderContext<'_>,
    ) -> Bounds {
        let mut bounds = self.bounds(render_buffer, context);
        bounds.width = self.get_gutter_width(context);
        bounds
    }

    fn draw_gutter(
        &self,
        buffer: &mut RenderBuffer,
        context: &mut RenderContext,
    ) -> anyhow::Result<()> {
        let top_line = context.viewport.top_line();
        let style = Style::from(context.theme.colors.gutter);
        let Bounds {
            start_col,
            width,
            height,
            ..
        } = self.get_gutter_bounds(buffer, context);
        let line_count = context.document.buffer.line_count();

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

    fn get_buffer_bounds(
        &self,
        render_buffer: &RenderBuffer,
        context: &RenderContext<'_>,
    ) -> Bounds {
        let gutter_width = self.get_gutter_width(context);
        let mut bounds = self.bounds(render_buffer, context);
        bounds.start_col += gutter_width;
        bounds.width -= gutter_width;
        bounds
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
        } = self.get_buffer_bounds(render_buffer, context);
        let viewport = context.viewport;
        let buffer = &context.document.buffer;
        let theme = context.theme;

        let top_line = viewport.top_line();
        let left_col = viewport.left_column();
        let editor_style = theme.editor_style();

        for viewport_row in 0..visible_height {
            let buffer_row = top_line + viewport_row;

            let content = if buffer_row >= buffer.line_count() {
                " ".repeat(visible_width)
            } else {
                let line = buffer.get_content_line(buffer_row);
                format!(
                    "{:<visible_width$}",
                    line.chars()
                        .skip(left_col)
                        .take_while(|c| c != &'\n')
                        .collect::<String>()
                )
            };

            render_buffer.set_text(viewport_row, start_col, &content, &editor_style);
        }

        Ok(())
    }

    fn render_with_syntax_highlighting(
        &self,
        render_buffer: &mut RenderBuffer,
        context: &mut RenderContext,
    ) -> Result<()> {
        let Bounds {
            start_col,
            width: visible_width,
            height: visible_height,
            ..
        } = self.get_buffer_bounds(render_buffer, context);

        let Some(ref mut syntax_engine) = context.document.syntax_engine else {
            return Err(anyhow::anyhow!("Syntax highlighter is not available"));
        };

        let viewport = context.viewport;
        let buffer = &context.document.buffer;
        let theme = context.theme;
        let editor_style = theme.editor_style();

        let code = buffer.to_bytes();
        let tokens = syntax_engine.highlight(&code)?;

        let top_line = viewport.top_line();
        let left_column = viewport.left_column();

        // Filter tokens to only those visible in the viewport
        let mut info_iter = tokens
            .iter()
            .filter(|info| {
                info.end_position.row >= top_line
                    && info.start_position.row < top_line + visible_height as usize
            })
            .map(|info| {
                let mut new_info = info.clone();
                new_info.start_position.row -= top_line;
                new_info.end_position.row -= top_line;
                new_info
            })
            .peekable();

        // Render the first unhighlighted part of the code
        let first = if let Some(info) = info_iter.peek() {
            &code[..info.byte_range.start]
        } else {
            &code
        };

        let mut lines = first.split(|&b| b == b'\n').skip(top_line).peekable();

        let mut position = tree_sitter::Point { row: 0, column: 0 };

        while let Some(line) = lines.next() {
            let text = from_utf8(line)?;

            for c in text.chars() {
                if position.column >= left_column {
                    render_buffer.set_cell(
                        position.row,
                        position.column - left_column + start_col,
                        c,
                        &editor_style,
                    );
                }
                position.column += 1;
            }

            if lines.peek().is_some() {
                render_buffer.set_text(
                    position.row,
                    position.column.saturating_sub(left_column).add(start_col),
                    &" ".repeat(visible_width),
                    &editor_style,
                );
                position.row += 1;
                position.column = 0;
            }
        }

        while let Some(info) = info_iter.next() {
            let style = context.theme.style_for_token(&info.scope);
            let bytes = &code[info.byte_range.start..info.byte_range.end];
            position.row = info.end_position.row;
            position.column = info.end_position.column;

            self.set_text_on_viewport(
                render_buffer,
                context,
                &mut Point {
                    row: info.start_position.row,
                    column: info.start_position.column,
                },
                bytes,
                &style,
            )?;

            match info_iter.peek() {
                // Next highlight on the same line
                Some(next) => {
                    if info.byte_range.end <= next.byte_range.start {
                        self.set_text_on_viewport(
                            render_buffer,
                            context,
                            &mut position,
                            &code[info.byte_range.end..next.byte_range.start],
                            &editor_style,
                        )?;
                    }
                }
                // Next highlight on the next line
                None => {
                    self.set_text_on_viewport(
                        render_buffer,
                        context,
                        &mut position,
                        &code[info.byte_range.end..],
                        &editor_style,
                    )?;
                }
            }
        }

        // Fill the remaining rows
        let empty = " ".repeat(visible_width as usize);
        while position.row < visible_height as usize {
            render_buffer.set_text(
                position.row,
                position.column.saturating_sub(left_column).add(start_col),
                &empty,
                &editor_style,
            );
            position.row += 1;
            position.column = 0;
        }

        Ok(())
    }

    fn set_text_on_viewport(
        &self,
        render_buffer: &mut RenderBuffer,
        context: &mut RenderContext,
        position: &mut Point,
        bytes: &[u8],
        style: &Style,
    ) -> Result<()> {
        let Bounds {
            start_col,
            width: visible_width,
            height,
            ..
        } = self.get_buffer_bounds(render_buffer, context);
        let left_column = context.viewport.left_column();

        let mut lines = bytes.split(|&c| c == b'\n').peekable();

        while let Some(line) = lines.next() {
            let text = from_utf8(line)?;

            for c in text.chars() {
                if position.column >= left_column {
                    render_buffer.set_cell(
                        position.row,
                        position.column - left_column + start_col,
                        c,
                        &style,
                    );
                }
                position.column += 1;
            }

            if lines.peek().is_some() {
                render_buffer.set_text(
                    position.row,
                    position.column.saturating_sub(left_column).add(start_col),
                    &" ".repeat(visible_width),
                    &style,
                );
                if position.row + 1 >= height as usize {
                    break;
                }
                position.row += 1;
                position.column = 0;
            }
        }
        Ok(())
    }

    fn draw_diagnostics(
        &self,
        render_buffer: &mut RenderBuffer,
        context: &mut RenderContext,
    ) -> Result<()> {
        let bounds = self.get_buffer_bounds(render_buffer, context);
        let buffer = &context.document.buffer;
        let viewport = context.viewport;
        let starting_line = viewport.top_line();
        let ending_line = starting_line + bounds.height;

        let visible_diagnostics = context
            .diagnostics
            .iter()
            .filter(|d| {
                let start = &d.range.start;
                start.line >= starting_line && start.line < ending_line
            })
            .filter(|d| d.severity <= DiagnosticSeverity::Warning);

        for diagnostic in visible_diagnostics {
            let Some(message) = diagnostic.message.lines().next() else {
                continue;
            };
            let formatted = format!("■  {message}");
            let line = diagnostic.range.start.line;
            let line_length = buffer.get_line_length(line);
            let column = line_length + DIAGNOSTIC_MARGIN;

            let formatted: String = formatted
                .chars()
                .skip(viewport.left_column().saturating_sub(column))
                .collect();

            let style = context.theme.get_diagnostic_style(&diagnostic.severity);

            render_buffer.set_text(
                line - starting_line,
                column
                    .saturating_sub(viewport.left_column())
                    .add(bounds.start_col),
                &formatted,
                &style,
            );
        }
        Ok(())
    }

    fn draw_buffer(
        &self,
        render_buffer: &mut RenderBuffer,
        context: &mut RenderContext,
    ) -> Result<()> {
        if context.document.language.is_plain_text() {
            return self.render_plain_text(render_buffer, context);
        }

        if let Err(e) = self.render_with_syntax_highlighting(render_buffer, context) {
            log::error!("Failed to render buffer with syntax highlighting: {}", e);
            self.render_plain_text(render_buffer, context)?;
        }

        Ok(())
    }
}

impl Drawable for EditorView {
    fn draw(&self, render_buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        self.draw_gutter(render_buffer, context)?;
        self.draw_buffer(render_buffer, context)?;
        self.draw_diagnostics(render_buffer, context)
    }

    fn bounds(&self, render_buffer: &RenderBuffer, _context: &RenderContext<'_>) -> Bounds {
        let width = render_buffer.width;
        let height = render_buffer.height - RESERVED_ROW_COUNT;
        Bounds {
            start_row: 0,
            start_col: 0,
            width,
            height,
        }
    }
}

impl Focusable for EditorView {
    fn get_display_cursor(&self, _: &RenderBuffer, context: &RenderContext) -> (usize, usize) {
        let viewport = context.viewport;
        let (row, column) = context.cursor.get_display_cursor();
        let gutter_width = self.get_gutter_width(context);
        let screen_row = row - viewport.top_line();
        let screen_col = column - viewport.left_column();
        (screen_row, screen_col + gutter_width)
    }
}
