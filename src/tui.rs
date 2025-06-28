use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, stdout};

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

pub fn init(is_inline: bool) -> Result<Tui> {
    if !is_inline {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
    }
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: if !is_inline {
                Viewport::Fullscreen
            } else {
                Viewport::Inline(5)
            },
        },
    )?;
    Ok(terminal)
}

pub fn restore(is_inline: bool, terminal: &mut Tui) -> Result<()> {
    if !is_inline {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    Ok(())
}
