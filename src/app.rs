use crate::{config::Config, events, state::AppState, tui::Tui, ui};
use color_eyre::Result;
use ratatui::layout::{Margin, Position};
use ratatui::style::Color;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration as StdDuration, Instant};
use tachyonfx::{fx, ref_count, CellFilter, Effect, EffectRenderer, Interpolation, Motion};

/// How much each ↑/↓ press changes WPM.
const WPM_STEP: i64 = 10;
/// How long to wait after the last ↑/↓ press before leaving adjust mode.
const ADJUST_IDLE: StdDuration = StdDuration::from_millis(800);
/// Poll interval while the help overlay blocks reading and no effects run.
const IDLE_POLL: StdDuration = StdDuration::from_millis(100);

/// Color-tint a freshly-rendered word in from the theme accent.
///
/// Duration scales with WPM so fast words aren't still fading when the next one
/// lands. The focus letter is excluded from the fade (it stays at full focus
/// color instantly) so the anchor is always readable.
///
/// `focus_x` is shared with the render loop and read at apply time, because
/// the effect is created *before* the new word is laid out and its focus
/// column may differ from the previous word's.
fn word_transition(accent: Color, wpm: u64, focus_x: Rc<Cell<u16>>) -> Effect {
    let window_ms = 60_000.0 / wpm.max(1) as f64;
    let fade_ms = (window_ms * 0.25).clamp(15.0, 90.0) as u32;

    fx::fade_from_fg(accent, (fade_ms, Interpolation::QuadOut)).with_filter(
        CellFilter::Not(Box::new(CellFilter::PositionFn(ref_count(
            move |p: Position| p.x == focus_x.get(),
        )))),
    )
}

pub fn run(
    content: &str,
    wpm: u64,
    preview_words: Option<usize>,
    config: &Config,
    terminal: &mut Tui,
) -> Result<()> {
    let mut app_state = AppState::new(content, wpm);

    let preview_count = preview_words.unwrap_or(config.preview_words);

    let render_opts = ui::RenderOptions {
        preview_count,
        border_color: config.parse_border_color(),
        progress_bar_color: config.parse_progress_bar_color(),
        focus_color: config.parse_focus_color(),
        show_border: config.show_border,
        show_progress_bar: config.show_progress_bar,
    };

    let seek_step = config.seek_step.max(1) as isize;
    let animations = config.enable_animations;

    // TachyonFX effects are stateful: create once, apply every frame after
    // the widgets render. Startup and word effects are one-shot; the progress
    // bar sweep runs indefinitely.
    //
    // Startup reveals only the border ring (CellFilter::Outer), so the first
    // word and progress bar are readable immediately instead of materializing.
    let accent = config.parse_border_color();
    let mut startup_fx: Option<Effect> = animations.then(|| {
        fx::coalesce((600, Interpolation::QuadOut))
            .with_filter(CellFilter::Outer(Margin::new(1, 1)))
    });
    let mut word_fx: Option<Effect> = None;
    let mut progress_fx: Option<Effect> = (animations && config.show_progress_bar).then(|| {
        fx::repeating(fx::sweep_in(
            Motion::LeftToRight,
            6,
            0,
            Color::DarkGray,
            (1500, Interpolation::Linear),
        ))
    });

    let mut last_frame = Instant::now();
    let mut last_word_idx = app_state.current_word_index();

    // Help overlay state
    let mut show_help = false;
    let mut help_scroll: u16 = 0;
    let help_border_color = config.parse_border_color();

    // Absolute x of the current focus letter, so the word effect can skip it.
    // Shared with the effect filter and refreshed every frame.
    let focus_x = Rc::new(Cell::new(0u16));

    loop {
        let now = Instant::now();
        let frame_dt = now - last_frame;
        last_frame = now;
        let dt: tachyonfx::Duration = frame_dt.into();

        terminal.draw(|f| {
            let areas = ui::render_word_display(f, &app_state, &render_opts);
            focus_x.set(areas.focus_x);

            if animations {
                if let Some(fx) = startup_fx.as_mut() {
                    f.render_effect(fx, areas.box_area, dt);
                }
                if let Some(fx) = word_fx.as_mut() {
                    f.render_effect(fx, areas.word, dt);
                }
                if let Some(fx) = progress_fx.as_mut() {
                    f.render_effect(fx, areas.progress, dt);
                }
            }

            // Render help popup on top of everything else
            if show_help {
                ui::render_help_popup(f, help_border_color, help_scroll, config.seek_step);
            }
        })?;

        // Drop one-shot effects once they finish.
        if startup_fx.as_ref().is_some_and(|fx| !fx.running()) {
            startup_fx = None;
        }
        if word_fx.as_ref().is_some_and(|fx| !fx.running()) {
            word_fx = None;
        }

        // Poll at a steady rate so effects animate smoothly. Word timing is
        // still driven by `should_advance()` against the wall clock. While the
        // help overlay is up the word is frozen, so don't let an elapsed
        // `next_tick` collapse the timeout to zero and spin the CPU.
        let timeout = if animations {
            StdDuration::from_millis(16)
        } else if show_help {
            IDLE_POLL
        } else {
            app_state.get_timeout()
        };

        match events::handle_events(timeout)? {
            events::AppEvent::Quit => {
                if show_help {
                    show_help = false;
                    help_scroll = 0;
                    app_state.restart_word_timer();
                } else {
                    break;
                }
            }
            events::AppEvent::TogglePause => app_state.toggle_pause(),
            events::AppEvent::ToggleHelp => {
                show_help = !show_help;
                if !show_help {
                    help_scroll = 0;
                    // Give the frozen word a full window instead of skipping
                    // it the instant the overlay closes.
                    app_state.restart_word_timer();
                }
            }
            events::AppEvent::ScrollDown => {
                if show_help {
                    help_scroll = help_scroll.saturating_add(1);
                }
            }
            events::AppEvent::ScrollUp => {
                if show_help {
                    help_scroll = help_scroll.saturating_sub(1);
                }
            }
            events::AppEvent::WpmUp => {
                if !show_help {
                    app_state.adjust_wpm(WPM_STEP);
                }
            }
            events::AppEvent::WpmDown => {
                if !show_help {
                    app_state.adjust_wpm(-WPM_STEP);
                }
            }
            events::AppEvent::FastForward => {
                if !show_help {
                    app_state.seek_word(seek_step);
                }
            }
            events::AppEvent::Rewind => {
                if !show_help {
                    app_state.seek_word(-seek_step);
                }
            }
            events::AppEvent::Continue => {}
        }

        // Leave adjust mode once the user stops nudging WPM for a moment.
        if app_state.should_end_adjust(ADJUST_IDLE) {
            app_state.end_adjust();
        }

        // Advance the word when its display window elapses.
        if !show_help && app_state.should_advance() && !app_state.advance_word() {
            break; // Reading complete
        }

        // Trigger a transition whenever the visible word changes (advance or seek).
        let new_idx = app_state.current_word_index();
        if animations && new_idx != last_word_idx {
            word_fx = Some(word_transition(
                accent,
                app_state.wpm(),
                Rc::clone(&focus_x),
            ));
        }
        last_word_idx = new_idx;
    }

    Ok(())
}
