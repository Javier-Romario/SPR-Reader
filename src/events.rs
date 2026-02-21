use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub enum AppEvent {
    Quit,
    TogglePause,
    ToggleHelp,
    ScrollUp,
    ScrollDown,
    Continue,
}

pub fn handle_events(timeout: Duration) -> Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(AppEvent::Quit),
                KeyCode::Char(' ') => return Ok(AppEvent::TogglePause),
                KeyCode::Char('?') => return Ok(AppEvent::ToggleHelp),
                KeyCode::Char('j') | KeyCode::Down => return Ok(AppEvent::ScrollDown),
                KeyCode::Char('k') | KeyCode::Up => return Ok(AppEvent::ScrollUp),
                _ => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}
