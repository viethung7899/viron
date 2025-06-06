mod buffer;
mod editor;
mod logger;

use buffer::Buffer;
use editor::Editor;
use logger::Logger;
use once_cell::sync::OnceCell;

fn main() -> anyhow::Result<()> {
    let file = std::env::args().nth(1);

    let buffer = file
        .map(|path| Buffer::from_file(&path))
        .unwrap_or_default();

    let mut editor = Editor::new(buffer)?;

    log!("{}", "Starting");
    editor.run()
}

static LOGGER: OnceCell<Logger> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        let message = format!($($args)*);
        $crate::LOGGER.get_or_init(|| $crate::Logger::new("target/debug/viron.log")).log(&message);
    };
}
