mod buffer;
mod editor;

use buffer::Buffer;
use editor::Editor;

fn main() -> anyhow::Result<()> {
    let file = std::env::args().nth(1);

    let buffer = file
        .map(|path| Buffer::from_file(&path))
        .unwrap_or_default();

    let mut editor = Editor::new(buffer)?;
    editor.run()
}
