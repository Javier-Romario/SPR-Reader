use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Quit,
    TogglePause,
    ToggleHelp,
    ScrollUp,
    ScrollDown,
    WpmUp,
    WpmDown,
    FastForward,
    Rewind,
    Continue,
}

pub fn handle_events(timeout: Duration) -> Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            // Windows reports Release/Repeat as well; only act on Press so a
            // single tap doesn't toggle twice.
            if key.kind != KeyEventKind::Press {
                return Ok(AppEvent::Continue);
            }
            // Raw mode swallows SIGINT, so Ctrl+C arrives as a key event.
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(AppEvent::Quit);
            }
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(AppEvent::Quit),
                KeyCode::Char(' ') => return Ok(AppEvent::TogglePause),
                KeyCode::Char('?') => return Ok(AppEvent::ToggleHelp),
                KeyCode::Char('j') => return Ok(AppEvent::ScrollDown),
                KeyCode::Char('k') => return Ok(AppEvent::ScrollUp),
                KeyCode::Up => return Ok(AppEvent::WpmUp),
                KeyCode::Down => return Ok(AppEvent::WpmDown),
                KeyCode::Char('l') | KeyCode::Right => return Ok(AppEvent::FastForward),
                KeyCode::Char('h') | KeyCode::Left => return Ok(AppEvent::Rewind),
                _ => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}
