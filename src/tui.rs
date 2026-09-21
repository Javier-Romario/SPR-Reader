use color_eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, stdout, IsTerminal};

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

pub fn init(is_inline: bool) -> Result<Tui> {
    #[cfg(unix)]
    redirect_stdin_if_piped()?;

    enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;
    if !is_inline {
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

// After piped stdin is drained, crossterm still polls fd 0 for key events.
// Replacing fd 0 with /dev/tty lets it reach the actual keyboard.
#[cfg(unix)]
fn redirect_stdin_if_piped() -> Result<()> {
    use std::os::unix::io::AsRawFd;
    if !io::stdin().is_terminal() {
        let tty = std::fs::File::open("/dev/tty").map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "spr needs a terminal: stdin was piped but no TTY (/dev/tty) is available",
            )
        })?;
        let ret = unsafe { libc::dup2(tty.as_raw_fd(), libc::STDIN_FILENO) };
        if ret == -1 {
            return Err(io::Error::last_os_error().into());
        }
    }
    Ok(())
}

pub fn restore(is_inline: bool, terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), cursor::Show)?;
    if !is_inline {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    Ok(())
}
