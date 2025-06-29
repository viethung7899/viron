use crate::core::buffer_manager::BufferManager;
use crate::core::cursor::Cursor;
use crate::core::viewport::Viewport;
use crate::editor::Mode;
use crate::ui::render_buffer::RenderBuffer;
use crate::ui::theme::Theme;

pub mod components;
pub mod compositor;
pub mod render_buffer;
pub mod theme;

pub struct RenderContext<'a> {
    pub viewport: &'a Viewport,
    pub buffer_manager: &'a mut BufferManager,
    pub cursor: &'a Cursor,
    pub mode: &'a Mode,
    pub theme: &'a Theme,
}

pub struct Bounds {
    pub start_row: usize,
    pub start_col: usize,
    pub width: usize,
    pub height: usize,
}

pub trait Drawable {
    fn id(&self) -> &str;
    fn draw(&self, buffer: &mut RenderBuffer, context: &mut RenderContext) -> anyhow::Result<()>;
    fn bounds(&self, size: (usize, usize), context: &RenderContext) -> Bounds;
}
