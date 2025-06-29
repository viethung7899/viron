mod config;
mod core;
mod editor;
mod input;
mod service;
mod ui;
mod utils;

use anyhow::Result;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal};
use editor::Editor;
use std::path::Path;
use std::{env, io::stdout, panic};
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable better panic messages
    better_panic::install();

    // Initialize logging if needed
    setup_log()?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Create and initialize editor
    let mut editor = Editor::new()?;

    // Load the config
    editor.load_config(&Config::load_from_file("config.v2.toml")?)?;

    // Set up error handling for the editor's run method
    let result = run_editor(&mut editor, &args);

    // Always clean up terminal state, even if run_editor fails
    if let Err(e) = editor.cleanup() {
        eprintln!("Error cleaning up terminal: {}", e);
    }

    panic::set_hook(Box::new(|info| {
        let mut stdout = stdout();
        _ = crossterm::execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::Show,
            terminal::LeaveAlternateScreen,
        );
        _ = terminal::disable_raw_mode();
        eprintln!("{}", info);
    }));

    // Return the result from run_editor
    result
}

fn setup_log() -> Result<()> {
    use env_logger::{Builder, Target};
    use std::fs::File;
    use log::LevelFilter;

    let file = File::create("/tmp/viron.log")?;
    Builder::new()
        .target(Target::Pipe(Box::new(file)))
        .filter(None, LevelFilter::Info)
        .init();

    Ok(())
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
