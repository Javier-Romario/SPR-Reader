use crate::{events, state::AppState, tui::Tui, ui};
use color_eyre::Result;
use std::io;

pub fn run<'a>(content: &'a str, wpm: u64, is_inline: bool, terminal: &mut Tui) -> Result<()> {
    if content.split_whitespace().collect::<Vec<_>>().is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No words to display").into());
    }

    let mut app_state = AppState::new(content, wpm);
    let ui_constraints = ui::UIConstraints::new(is_inline);

    loop {
        terminal.draw(|f| {
            ui::render_word_display(f, app_state.current_word(), &ui_constraints);
        })?;

        let timeout = app_state.get_timeout();
        
        match events::handle_events(timeout)? {
            events::AppEvent::Quit => break,
            events::AppEvent::TogglePause => app_state.toggle_pause(),
            events::AppEvent::Continue => {}
        }

        if app_state.should_advance() {
            app_state.advance_word();
        }
    }

    Ok(())
}
