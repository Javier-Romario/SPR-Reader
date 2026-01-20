mod app;
mod cli;
mod events;
mod state;
mod tui;
mod ui;

use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();

    let content = cli::get_content(&args)?;

    // Validate content before initializing TUI
    if content.split_whitespace().collect::<Vec<_>>().is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "No words to display").into(),
        );
    }

    let is_inline = args.inline.unwrap_or(false);

    let mut terminal = tui::init(is_inline)?;

    app::run(&content, args.wpm, is_inline, &mut terminal)?;

    tui::restore(is_inline, &mut terminal)?;

    Ok(())
}
