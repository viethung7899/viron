mod buffer;
mod config;
mod editor;
mod highlighter;
mod logger;
mod lsp;
mod theme;
mod utils;


use std::{io::stdout, panic};

use config::Config;
use crossterm::{ExecutableCommand, event, terminal};
use editor::Editor;
use logger::Logger;
use once_cell::sync::OnceCell;

use crate::lsp::LspClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let toml = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&toml)?;

    let file = std::env::args().nth(1);
    let theme = theme::parse_vscode_theme(&config.theme)?;

    if let Some(log_file) = &config.log_file {
        LOGGER.get_or_init(|| Some(Logger::new(log_file)));
    } else {
        LOGGER.get_or_init(|| None);
    }

    let mut lsp = LspClient::start().await?;
    lsp.initialize().await?;

    let mut editor = Editor::new(config, theme, lsp, file).await?;

    panic::set_hook(Box::new(|info| {
        let mut stdout = stdout();
        _ = stdout.execute(terminal::LeaveAlternateScreen);
        _ = stdout.execute(event::DisableMouseCapture);
        _ = terminal::disable_raw_mode();
        eprintln!("{}", info);
    }));

    editor.run().await
}

#[allow(unused)]
static LOGGER: OnceCell<Option<Logger>> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        let message = format!($($args)*);
        if let Some(logger) = $crate::LOGGER.get_or_init(|| Some($crate::Logger::new("target/debug/viron.log"))) {
            logger.log(&message);
        }
    };
}
