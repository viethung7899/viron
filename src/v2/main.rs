mod config;
mod core;
mod editor;
mod input;
mod service;
mod ui;
mod utils;

use crate::config::Config;
use anyhow::Result;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal};
use editor::Editor;
use std::{env, io::stdout, panic};

#[tokio::main]
async fn main() -> Result<()> {
    // Enable better panic messages
    better_panic::install();

    // Initialize logging if needed
    setup_log()?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let mut editor = Editor::new()?;

    // Load the config
    editor.load_config(&Config::load_from_file("config.v2.toml")?)?;

    if let Some(file_name) = args.get(1) {
        // If a file path is provided, create a new editor with that file
        editor.load_file(file_name).await?;
    }

    // Set up error handling for the editor's run method
    let result = editor.run().await;

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
    use log::LevelFilter;
    use std::fs::File;

    let file = File::create("/tmp/viron.log")?;
    Builder::new()
        .target(Target::Pipe(Box::new(file)))
        .filter(None, LevelFilter::Info)
        .init();

    Ok(())
}
