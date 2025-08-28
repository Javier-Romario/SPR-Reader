use crossterm::event::{self, Event, KeyCode};
use color_eyre::Result;
use std::time::Duration;

pub enum AppEvent {
    Quit,
    TogglePause,
    Continue,
}

pub fn handle_events(timeout: Duration) -> Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(AppEvent::Quit),
                KeyCode::Char(' ') => return Ok(AppEvent::TogglePause),
                _ => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}