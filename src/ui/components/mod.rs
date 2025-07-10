mod buffer_view;
mod command_line;
mod editor_view;
mod gutter;
mod message_area;
mod pending_keys;
mod search_box;
mod status_line;

pub use command_line::CommandLine;
pub use editor_view::EditorView;
pub use message_area::MessageArea;
pub use pending_keys::PendingKeys;
pub use search_box::SearchBox;
pub use status_line::StatusLine;

pub use editor_view::MIN_GUTTER_SIZE;

use crate::ui::Drawable;

pub struct ComponentIds {
    pub status_line_id: String,
    pub editor_view_id: String,

    pub pending_keys_id: String,
    pub command_line_id: String,
    pub message_area_id: String,
    pub search_box_id: String,
}

pub struct Component {
    pub dirty: bool,
    pub visible: bool,
    pub(in crate::ui) drawable: Box<dyn Drawable>,
}

impl Component {
    pub fn new(drawable: Box<dyn Drawable>) -> Self {
        Self {
            dirty: true,
            visible: true,
            drawable,
        }
    }

    pub fn new_invisible(drawable: Box<dyn Drawable>) -> Self {
        Self {
            dirty: true,
            visible: false,
            drawable,
        }
    }
}
