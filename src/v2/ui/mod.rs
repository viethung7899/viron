use crate::core::buffer::Buffer;
use crate::core::cursor::Cursor;
use crate::core::document::Document;
use crate::core::viewport::Viewport;
use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Theme;

pub mod command_line;
pub mod compositor;
pub mod gutter;
pub mod renderer;
pub mod status_line;
pub mod theme;
pub mod render_buffer;

pub struct RenderContext<'a> {
    pub viewport: &'a Viewport,
    pub document: &'a Document,
    pub cursor: &'a Cursor,
    pub mode: &'a Mode,
    pub theme: &'a Theme,
}

pub trait Drawable {
    fn id(&self) -> &str;
    fn draw(&self, buffer: &mut RenderBuffer, context: &RenderContext);
}
