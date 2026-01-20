use crate::{events, state::AppState, tui::Tui, ui};
use color_eyre::Result;
use std::time::Instant;
use tachyonfx::{fx, Duration as FxDuration, EffectManager, Interpolation};

pub fn run(content: &str, wpm: u64, is_inline: bool, terminal: &mut Tui) -> Result<()> {
    let mut last_frame = Instant::now();
    let mut effects: EffectManager<()> = EffectManager::default();
    let mut app_state = AppState::new(content, wpm);
    let fx = fx::coalesce((8000, Interpolation::SineInOut));
    let ui_constraints = ui::UIConstraints::new(is_inline);

    // Add startup effect (runs once at beginning)
    effects.add_effect(fx);
    //     .push(
    //     fx::coalesce((800, Interpolation::SineInOut))
    // );

    loop {
        terminal.draw(|f| {
            let screen_area = f.area();
            let elapsed = last_frame.elapsed();
            let duration_ms = FxDuration::from_millis(elapsed.as_millis() as u32);

            effects.process_effects(
                duration_ms,
                f.buffer_mut(),
                screen_area, // or specific area
            );

            ui::render_word_display(
                f,
                app_state.current_word(),
                app_state.current_word_index(),
                app_state.total_words(),
                app_state.is_paused(),
                &ui_constraints,
            );
        })?;

        let timeout = app_state.get_timeout();

        match events::handle_events(timeout)? {
            events::AppEvent::Quit => break,
            events::AppEvent::TogglePause => app_state.toggle_pause(),
            events::AppEvent::Continue => {}
        }

        if app_state.should_advance() && !app_state.advance_word() {
            break; // Reading complete
        }

        // Reset frame timer for next iteration
        last_frame = Instant::now();
    }

    Ok(())
}
