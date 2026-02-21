use crate::{config::Config, events, state::AppState, tui::Tui, ui};
use color_eyre::Result;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::time::Instant;
use tachyonfx::{Duration as FxDuration, EffectManager};

/// Adds a sweeping scanner effect to the progress bar
/// Creates a cyberpunk-style scanning beam that moves across the progress bar
/// Uses terminal-aware brightening to respect the color scheme
fn add_progress_scanner_effect(buffer: &mut Buffer, area: Rect, time_ms: u64) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Scanner sweeps across every 2.5 seconds
    let sweep_duration = 2500.0;
    let sweep_progress = (time_ms as f64 % sweep_duration) / sweep_duration;

    // Scanner position (moves left to right)
    let scanner_x = area.x + (sweep_progress * area.width as f64) as u16;

    // Scanner beam width (5 characters wide with intensity falloff for smoother effect)
    if scanner_x >= area.x && scanner_x < area.x + area.width {
        for y in area.y..area.y + area.height {
            // Apply glow to center and surrounding cells
            for offset in -2i16..=2 {
                let x = scanner_x as i16 + offset;
                if x >= area.x as i16 && x < (area.x + area.width) as i16 {
                    if let Some(cell) = buffer.cell_mut((x as u16, y)) {
                        // Gaussian-like falloff for smooth beam
                        let distance = offset.abs() as f64;
                        let intensity = (-distance * distance / 2.0).exp();

                        // Brighten by switching to lighter variants
                        if intensity > 0.6 {
                            // Core of beam - use White for maximum brightness
                            cell.set_fg(Color::White);
                        } else if intensity > 0.2 {
                            // Medium glow - try to brighten the existing color
                            let current_fg = cell.fg;
                            let brightened = brighten_color(current_fg);
                            cell.set_fg(brightened);
                        }
                    }
                }
            }
        }
    }
}

/// Brightens a color to its lighter variant (terminal-aware)
fn brighten_color(color: Color) -> Color {
    match color {
        Color::Black => Color::DarkGray,
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::Gray => Color::White,
        Color::DarkGray => Color::Gray,
        // Already light colors stay white
        Color::LightRed | Color::LightGreen | Color::LightYellow |
        Color::LightBlue | Color::LightMagenta | Color::LightCyan | Color::White => Color::White,
        // RGB colors get brightened by adding white
        Color::Rgb(r, g, b) => {
            Color::Rgb(
                r.saturating_add(80),
                g.saturating_add(80),
                b.saturating_add(80),
            )
        }
        _ => color,
    }
}

pub fn run(content: &str, wpm: u64, is_inline: bool, preview_words: Option<usize>, terminal: &mut Tui) -> Result<()> {
    let mut last_frame = Instant::now();
    let mut effects: EffectManager<()> = EffectManager::default();
    let mut app_state = AppState::new(content, wpm);

    // Load config
    let config = Config::load()?;
    let preview_count = preview_words.unwrap_or(config.preview_words);

    let ui_constraints = ui::UIConstraints::new(is_inline, preview_count);
    let is_first_use = Config::is_first_use()?;

    // Extract config values
    let border_color = if is_inline && config.show_border {
        Some(config.parse_border_color())
    } else {
        None
    };
    let progress_bar_color = config.parse_progress_bar_color();
    let focus_color = config.parse_focus_color();
    let enable_animations = config.enable_animations;
    let show_border = config.show_border;
    let show_progress_bar = config.show_progress_bar;
    let seek_step = config.seek_step as isize;

    // Border animation setup (only if animations are enabled)
    let border_animation_duration_ms = 600.0; // 0.6 seconds for full animation
    let animation_start = Instant::now();
    let should_animate_border = is_inline && border_color.is_some() && enable_animations;

    // Track total elapsed time for animations
    let session_start = Instant::now();

    // Help overlay state
    let mut show_help = false;
    let mut help_scroll: u16 = 0;
    let help_border_color = config.parse_border_color();

    // Mark first use as complete
    if is_first_use && is_inline {
        Config::mark_first_use_complete()?;
    }

    loop {
        terminal.draw(|f| {
            let screen_area = f.area();
            let elapsed = last_frame.elapsed();
            let duration_ms = FxDuration::from_millis(elapsed.as_millis() as u32);
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

            // Render UI and get progress bar area for effects
            let preview = app_state.peek_words(preview_count);
            let progress_area = ui::render_word_display(
                f,
                app_state.current_word().unwrap_or(""),
                &preview,
                app_state.current_word_index(),
                app_state.total_words(),
                app_state.is_paused(),
                &ui_constraints,
                is_inline,
                border_color,
                border_progress,
                time_ms,
                progress_bar_color,
                focus_color,
                enable_animations,
                show_border,
                show_progress_bar,
            );

            // Apply scanner sweep effect to progress bar (only if animations enabled)
            if enable_animations && show_progress_bar {
                add_progress_scanner_effect(f.buffer_mut(), progress_area, time_ms);
            }

            // Process all effects (border animations, etc.)
            effects.process_effects(
                duration_ms,
                f.buffer_mut(),
                screen_area,
            );

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

        // Check if border animation is complete
        let animation_complete = if should_animate_border {
            let elapsed_ms = animation_start.elapsed().as_millis() as f32;
            elapsed_ms >= border_animation_duration_ms
        } else {
            true // No animation, proceed immediately
        };

        // Only advance words after animation completes and help is not shown
        if animation_complete && !show_help && app_state.should_advance() && !app_state.advance_word() {
            break; // Reading complete
        }

        // Reset frame timer for next iteration
        last_frame = Instant::now();
    }

    Ok(())
}
