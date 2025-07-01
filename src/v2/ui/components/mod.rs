mod buffer_view;
mod command_line;
mod gutter;
mod status_line;

pub use buffer_view::BufferView;
pub use command_line::CommandLine;
pub use gutter::Gutter;
pub use status_line::StatusLine;
use std::hash::Hash;

use crate::ui::Drawable;

pub struct ComponentIds {
    pub status_line_id: String,
    pub gutter_id: String,
    pub buffer_view_id: String,
    pub command_line_id: String,
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

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Component {}
impl Hash for Component {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
