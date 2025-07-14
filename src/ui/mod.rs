use crate::config::Config;
use crate::core::command::{CommandBuffer, SearchBuffer};
use crate::core::cursor::Cursor;
use crate::core::message::MessageManager;
use crate::core::viewport::Viewport;
use crate::ui::render_buffer::RenderBuffer;

pub(crate) mod components;
pub mod compositor;
pub mod render_buffer;
pub mod theme;

use crate::core::document::Document;
use crate::core::mode::Mode;
use crate::input::InputState;
use crate::service::lsp::types::Diagnostic;

pub struct RenderContext<'a> {
    pub viewport: &'a Viewport,
    pub document: &'a mut Document,
    pub diagnostics: &'a [Diagnostic],
    pub cursor: &'a Cursor,
    pub mode: &'a Mode,
    pub config: &'a Config,
    pub command_buffer: &'a CommandBuffer,
    pub search_buffer: &'a SearchBuffer,
    pub message_manager: &'a MessageManager,
    pub input_state: &'a InputState,
}

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
