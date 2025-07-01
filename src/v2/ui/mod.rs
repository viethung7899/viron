use crate::core::buffer_manager::BufferManager;
use crate::core::command_buffer::CommandBuffer;
use crate::core::cursor::Cursor;
use crate::core::message::MessageManager;
use crate::core::viewport::Viewport;
use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Theme;

pub(crate) mod components;
pub mod compositor;
pub mod render_buffer;
pub mod theme;

use crate::input::keymaps::KeySequence;
pub use components::Component;

pub struct RenderContext<'a> {
    pub viewport: &'a Viewport,
    pub buffer_manager: &'a mut BufferManager,
    pub cursor: &'a Cursor,
    pub mode: &'a Mode,
    pub theme: &'a Theme,
    pub command_buffer: &'a CommandBuffer,
    pub message_manager: &'a MessageManager,
    pub pending_keys: &'a KeySequence,
    pub gutter_width: usize,
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
                &context.theme.editor_style(),
            );
        }

        Ok(())
    }
}
