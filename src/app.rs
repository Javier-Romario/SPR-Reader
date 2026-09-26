use crate::{config::Config, events, state::AppState, tui::Tui, ui};
use color_eyre::Result;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::time::Instant;

/// Adds a sweeping scanner effect to the progress bar.
/// A beam sweeps left-to-right every 2.5s, brightening the cells it passes.
fn add_progress_scanner_effect(buffer: &mut Buffer, area: Rect, time_ms: u64) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let sweep_duration = 2500.0;
    let sweep_progress = (time_ms as f64 % sweep_duration) / sweep_duration;
    let scanner_x = area.x + (sweep_progress * area.width as f64) as u16;

    if scanner_x >= area.x && scanner_x < area.x + area.width {
        for y in area.y..area.y + area.height {
            // Gaussian-like intensity falloff across a 5-cell beam
            for offset in -2i16..=2 {
                let x = scanner_x as i16 + offset;
                if x >= area.x as i16 && x < (area.x + area.width) as i16 {
                    if let Some(cell) = buffer.cell_mut((x as u16, y)) {
                        let distance = offset.abs() as f64;
                        let intensity = (-distance * distance / 2.0).exp();
                        if intensity > 0.6 {
                            cell.set_fg(Color::White);
                        } else if intensity > 0.2 {
                            cell.set_fg(ui::brighten_color(cell.fg));
                        }
                    }
                }
            }
        }
    }
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

    let border_color = if config.show_border {
        Some(config.parse_border_color())
    } else {
        None
    };

    let render_opts = ui::RenderOptions {
        preview_count,
        border_color,
        progress_bar_color: config.parse_progress_bar_color(),
        focus_color: config.parse_focus_color(),
        enable_animations: config.enable_animations,
        show_border: config.show_border,
        show_progress_bar: config.show_progress_bar,
    };

    let seek_step = config.seek_step.max(1) as isize;

    // Border animation setup (only if animations are enabled and a border is drawn)
    let border_animation_duration_ms = 600.0; // 0.6 seconds for full animation
    let animation_start = Instant::now();
    let should_animate_border = border_color.is_some() && config.enable_animations;

    // Track total elapsed time for animations
    let session_start = Instant::now();

    // Help overlay state
    let mut show_help = false;
    let mut help_scroll: u16 = 0;
    let help_border_color = config.parse_border_color();

    loop {
        terminal.draw(|f| {
            let time_ms = session_start.elapsed().as_millis() as u64;

            // Calculate border animation progress
            let border_progress = if should_animate_border {
                let elapsed_ms = animation_start.elapsed().as_millis() as f32;
                let progress = (elapsed_ms / border_animation_duration_ms).min(1.0);
                if progress < 1.0 {
                    Some(progress)
                } else {
                    None // Animation complete, use normal border
                }
            } else {
                None
            };

            let progress_area = ui::render_word_display(
                f,
                &app_state,
                &render_opts,
                border_progress,
                time_ms,
            );

            // Apply scanner sweep effect to progress bar (only if animations enabled)
            if config.enable_animations && config.show_progress_bar {
                add_progress_scanner_effect(f.buffer_mut(), progress_area, time_ms);
            }

            // Render help popup on top of everything else
            if show_help {
                ui::render_help_popup(f, help_border_color, help_scroll, config.seek_step);
            }
        })?;

        let timeout = app_state.get_timeout();

        match events::handle_events(timeout)? {
            events::AppEvent::Quit => {
                if show_help {
                    show_help = false;
                    help_scroll = 0;
                } else {
                    break;
                }
            }
            events::AppEvent::TogglePause => app_state.toggle_pause(),
            events::AppEvent::ToggleHelp => {
                show_help = !show_help;
                if !show_help {
                    help_scroll = 0;
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

        // Only advance words after the border animation completes and help is hidden
        let animation_complete = if should_animate_border {
            animation_start.elapsed().as_millis() as f32 >= border_animation_duration_ms
        } else {
            true // No animation, proceed immediately
        };

        if animation_complete
            && !show_help
            && app_state.should_advance()
            && !app_state.advance_word()
        {
            break; // Reading complete
        }
    }

    Ok(())
}
