mod core;
mod editor;
mod input;
mod service;
mod ui;
mod utils;

use anyhow::Result;
use editor::Editor;
use std::env;
use std::path::Path;
use std::process;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable better panic messages
    better_panic::install();

    // Initialize logging if needed

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Create and initialize editor
    let mut editor = Editor::new()?;

    // Load the config
    editor.load_config("config.v2.toml")?;

    // Set up error handling for the editor's run method
    let result = run_editor(&mut editor, &args);

    // Always clean up terminal state, even if run_editor fails
    if let Err(e) = editor.cleanup() {
        eprintln!("Error cleaning up terminal: {}", e);
    }

    // Return the result from run_editor
    result
}

fn run_editor(editor: &mut Editor, args: &[String]) -> Result<()> {
    // Open file if provided in arguments
    if args.len() > 1 {
        let path = &args[1];
        if let Err(e) = editor.buffer_manager_mut().open_file(Path::new(path)) {
            eprintln!("Error opening file {}: {}", path, e);
            // Continue and open an empty buffer
        }
    }

    // Run the editor's main loop
    editor.run()?;

    Ok(())
}
