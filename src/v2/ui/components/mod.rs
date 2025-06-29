mod status_line;
mod gutter;
mod buffer_view;

pub use status_line::StatusLine;
pub use gutter::Gutter;
pub use buffer_view::BufferView;

pub struct ComponentIds {
    pub status_line_id: String,
    pub gutter_id: String,
    pub buffer_view_id: String,
}
