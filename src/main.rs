mod config;
mod constants;
mod core;
mod editor;
mod input;
mod service;
mod ui;
mod actions;

use crate::config::{get_config_dir, Config};
use anyhow::Result;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal};
use std::{env, io::stdout, panic};
use crossterm::cursor::SetCursorStyle;
use crate::editor::EditorBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable better panic messages
    better_panic::install();

    // Initialize logging if needed
    setup_log()?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let file_name = args.get(1);

    let config_path = get_config_dir().join("config.toml");
    let config = Config::load_from_file(config_path)?;

    // Build the editor
    let mut builder = EditorBuilder::new()
        .with_config(config);

    if let Some(file) = file_name {
        builder = builder.with_file(file);
    }
    let mut editor = builder.build().await?;

    // Set up error handling for the editor's run method
    let result = editor.run().await;

    // Always clean up terminal state, even if run_editor fails
    if let Err(e) = editor.cleanup().await {
        log::error!("Error cleaning up terminal: {}", e);
    }

    panic::set_hook(Box::new(|info| {
        let mut stdout = stdout();
        _ = crossterm::execute!(
            stdout,
            terminal::Clear(ClearType::All),
            SetCursorStyle::DefaultUserShape,
            cursor::Show,
            terminal::LeaveAlternateScreen,
        );
        _ = terminal::disable_raw_mode();
        log::error!("{}", info);
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
