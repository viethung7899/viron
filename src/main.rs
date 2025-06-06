mod buffer;
mod editor;
mod logger;
mod theme;

use std::{io::stdout, panic};

use buffer::Buffer;
use crossterm::{ExecutableCommand, terminal};
use editor::Editor;
use logger::Logger;
use once_cell::sync::OnceCell;

fn main() -> anyhow::Result<()> {
    let file = std::env::args().nth(1);

    let buffer = file
        .map(|path| Buffer::from_file(&path))
        .unwrap_or_default();

    let theme = theme::parse_vscode_theme("themes/catppuchin/frappe.json")?;

    let mut editor = Editor::new(theme, buffer)?;

    panic::set_hook(Box::new(|info| {
        _ = stdout().execute(terminal::LeaveAlternateScreen);
        _ = terminal::disable_raw_mode();
        eprintln!("{}", info);
    }));

    editor.run()
}

#[allow(unused)]
static LOGGER: OnceCell<Logger> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        let message = format!($($args)*);
        $crate::LOGGER.get_or_init(|| $crate::Logger::new("target/debug/viron.log")).log(&message);
    };
}
