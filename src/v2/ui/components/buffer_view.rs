use std::str::from_utf8;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::{Bounds, Drawable, RenderContext};
use anyhow::{Ok, Result};
use log::info;
use tree_sitter::Point;
use crate::ui::theme::Style;

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
        } = self.bounds(render_buffer.get_size(), context);

        let viewport = context.viewport;
        let buffer = context.buffer_manager.current_buffer();
        let theme = context.theme;
        let editor_style = theme.editor_style();

        let code = buffer.to_bytes();
        let tokens = context.buffer_manager.get_syntax_highlighter().highlight(&code)?;
        
        info!("{tokens:?}");

        let top_line = viewport.top_line();
        let left_column = viewport.left_column();

        // Filter tokens to only those visible in the viewport
        let mut info_iter = tokens
            .iter()
            .filter(|info| {
                info.end_position.row >= top_line && info.start_position.row < top_line + visible_height as usize
            })
            .map(|info| {
                let mut new_info = info.clone();
                new_info.start_position.row -= top_line;
                new_info.end_position.row -= top_line;
                new_info
            })
            .peekable();

        let mut position = tree_sitter::Point { row: 0, column: 0 };

        // Render the first unhighlighted part of the code
        let first = if let Some(info) = info_iter.peek() {
            &code[..info.byte_range.start]
        } else {
            &code
        };

        let mut lines = first
            .split(|&b| b == b'\n')
            .skip(top_line)
            .peekable();

        while let Some(line) = lines.next() {
            let text = from_utf8(line)?;
            if lines.peek().is_some() {
                let formatted = format!("{text:<w$}", w = visible_width as usize);
                render_buffer.set_text(
                    position.row,
                    position.column + start_col,
                    &formatted,
                    &editor_style,
                );
                if position.row + 1 >= visible_height {
                    break;
                }
                position.row += 1;
                position.column = 0;
            } else {
                render_buffer.set_text(
                    position.row,
                    position.column + start_col,
                    text,
                    &editor_style,
                );
                position.column += text.len();
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
                position.column + start_col,
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
        let Bounds { start_col: gutter, width, height, .. }
            = self.bounds(render_buffer.get_size(), context);

        let mut lines = bytes.split(|&c| c == b'\n').peekable();

        while let Some(line) = lines.next() {
            let text = from_utf8(&line)?;
            if lines.peek().is_some() {
                let text = format!("{text:<w$}", w = width as usize);
                render_buffer.set_text(position.row, position.column + gutter, &text, style);
                if position.row + 1 >= height as usize {
                    break;
                }
                position.row += 1;
                position.column = 0;
            } else {
                render_buffer.set_text(position.row, position.column + gutter, text, style);
                position.column += text.len();
            }
        }
        Ok(())
    }
}

impl Drawable for BufferView {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> Result<()> {
        if context.buffer_manager.current().language.is_plain_text() {
            self.render_plain_text(buffer, context)
        } else {
            self.render_with_syntax_highlighting(buffer, context)
        }
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
