mod config;
mod constants;
mod core;
mod editor;
mod input;
mod service;
mod ui;

use crate::config::{get_config_dir, Config};
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
    let file_name = args.get(1);

    let mut editor = Editor::new(file_name).await?;

    // Load the config
    let config_path = get_config_dir().join("config.toml");
    editor.load_config(&Config::load_from_file(config_path)?)?;

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
