mod app;
mod cli;
mod config;
mod events;
mod state;
mod tui;
mod ui;

use clap::Parser;
use color_eyre::Result;
use crossterm::execute;
use std::io::stdout;

fn main() -> Result<()> {
    color_eyre::install()?;
    install_panic_hook();

    let args = cli::Args::parse();
    let config = config::Config::load()?;

    let content = cli::get_content(&args)?;

    // Validate content before initializing TUI
    if content.split_whitespace().next().is_none() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "No words to display").into(),
        );
    }

    // Use CLI arg if provided, otherwise use config value
    let is_inline = args.inline.unwrap_or(config.inline);

    let mut terminal = tui::init(is_inline)?;

    app::run(
        &content,
        args.wpm,
        args.preview_words,
        &config,
        &mut terminal,
    )?;

    tui::restore(is_inline, &mut terminal)?;

    Ok(())
}

/// Restore the terminal even if a panic unwinds past `tui::restore`.
/// Prevents leaving the user stranded in raw mode / alternate screen.
fn install_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = execute!(
            stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        hook(info);
    }));
}
