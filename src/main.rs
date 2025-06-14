mod buffer;
mod config;
mod editor;
mod highlighter;
mod logger;
mod lsp;
mod theme;

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
static LOGGER: OnceCell<Logger> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        let message = format!($($args)*);
        $crate::LOGGER.get_or_init(|| $crate::Logger::new("target/debug/viron.log")).log(&message);
    };
}
