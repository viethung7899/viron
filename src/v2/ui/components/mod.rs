mod buffer_view;
mod command_line;
mod gutter;
mod message_area;
mod status_line;

pub use buffer_view::BufferView;
pub use command_line::CommandLine;
pub use gutter::Gutter;
pub use status_line::StatusLine;
pub use message_area::MessageArea;

use crate::ui::Drawable;

pub struct ComponentIds {
    pub status_line_id: String,
    pub gutter_id: String,
    pub buffer_view_id: String,
    pub command_line_id: String,
    pub message_area_id: String,
}

pub struct Component {
    pub id: String,
    pub dirty: bool,
    pub visible: bool,
    pub(in crate::ui) drawable: Box<dyn Drawable>,
}

impl Component {
    pub fn new(id: &str, drawable: Box<dyn Drawable>) -> Self {
        Self {
            id: id.to_string(),
            dirty: true,
            visible: true,
            drawable,
        }
    }
}
