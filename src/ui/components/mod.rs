mod command_line;
mod editor_view;
mod gutter;
mod message_area;
mod pending_keys;
mod search_box;
mod status_line;

use std::rc::Rc;

pub use command_line::CommandLine;
pub use editor_view::EditorView;
pub use message_area::MessageArea;
pub use pending_keys::PendingKeys;
pub use search_box::SearchBox;
pub use status_line::StatusLine;

use crate::ui::{Drawable, Focusable};

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
    pub(in crate::ui) drawable: Rc<dyn Drawable>,
    pub(in crate::ui) focusable: Option<Rc<dyn Focusable>>,
}
