mod app;
mod cli;
mod config;
mod events;
mod state;
mod tui;
mod ui;

use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    let config = config::Config::load()?;

    let content = cli::get_content(&args)?;

    // Validate content before initializing TUI
    if content.split_whitespace().collect::<Vec<_>>().is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "No words to display").into(),
        );
    }

    // Use CLI arg if provided, otherwise use config value
    let is_inline = args.inline.unwrap_or(config.inline);

    let mut terminal = tui::init(is_inline)?;

    app::run(&content, args.wpm, is_inline, &mut terminal)?;

    tui::restore(is_inline, &mut terminal)?;

    Ok(())
}
