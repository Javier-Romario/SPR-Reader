use crate::state::AppState;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::*,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, LineGauge, Paragraph},
};
use unicode_width::UnicodeWidthStr;

/// Theme / layout settings bundled together so `render_word_display` doesn't
/// need a dozen positional arguments.
pub struct RenderOptions {
    pub preview_count: usize,
    pub border_color: Option<Color>,
    pub progress_bar_color: Color,
    pub focus_color: Color,
    pub enable_animations: bool,
    pub show_border: bool,
    pub show_progress_bar: bool,
}

/// Draws a border progressively with cyberpunk effects (0.0 to 1.0)
/// Sequence: left verticals -> top/bottom verticals -> corners (with flash) -> horizontals (with glow trail)
fn draw_progressive_border(buf: &mut Buffer, area: Rect, progress: f32, color: Color) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let max_x = area.right().saturating_sub(1);
    let max_y = area.bottom().saturating_sub(1);
    let mid_y = area.y + area.height / 2;

    // Total animation stages
    let total_stages = 4.0 + (area.width as f32 - 2.0);
    let current_stage = (progress * total_stages).floor() as usize;
    let stage_progress = (progress * total_stages).fract();

    // Stage 0-1: Draw left edge verticals (subtle, no glow)
    // Center vertical (always drawn first)
    buf[(area.x, mid_y)].set_symbol("┃").set_fg(color);

    if current_stage >= 1 {
        // Top third and bottom third
        let third_y = area.y + area.height / 3;
        let two_third_y = area.y + (area.height * 2) / 3;

        buf[(area.x, third_y)].set_symbol("┃").set_fg(color);
        buf[(area.x, two_third_y)].set_symbol("┃").set_fg(color);
    }

    // Stage 2: Add vertical bars on top and bottom edges
    if current_stage >= 2 {
        let mid_x = area.x + area.width / 2;

        buf[(mid_x, area.y)].set_symbol("┃").set_fg(color);
        buf[(mid_x, max_y)].set_symbol("┃").set_fg(color);
    }

    // Stage 3: Add corners (subtle, no flash)
    if current_stage >= 3 {
        // Use special corner characters for clean look
        buf[(area.x, area.y)].set_symbol("╔").set_fg(color);
        buf[(max_x, area.y)].set_symbol("╗").set_fg(color);
        buf[(area.x, max_y)].set_symbol("╚").set_fg(color);
        buf[(max_x, max_y)].set_symbol("╝").set_fg(color);
    }

    // Stage 4+: Draw horizontal lines progressively (subtle trailing glow)
    if current_stage >= 4 {
        let horiz_chars = (area.width as f32 - 2.0) as usize;
        let chars_to_draw = if current_stage >= 4 + horiz_chars {
            horiz_chars
        } else {
            let base = current_stage - 4;
            base + (stage_progress > 0.0) as usize
        };

        // Draw top and bottom edges with very subtle glow trail
        for i in 0..chars_to_draw.min(horiz_chars) {
            let x = area.x + 1 + i as u16;

            // Calculate distance from drawing head for subtle glow effect
            let distance_from_head = chars_to_draw.saturating_sub(i);
            let line_color = if distance_from_head == 0 {
                // Only the drawing head gets subtle brightening
                brighten_color(color)
            } else {
                color
            };

            buf[(x, area.y)].set_symbol("═").set_fg(line_color);
            buf[(x, max_y)].set_symbol("═").set_fg(line_color);
        }
    }

    // Fill remaining vertical edges with double-line style
    if current_stage >= 4 {
        for y in area.y + 1..max_y {
            if y != mid_y && y != area.y + area.height / 3 && y != area.y + (area.height * 2) / 3 {
                buf[(area.x, y)].set_symbol("║").set_fg(color);
            }
            buf[(max_x, y)].set_symbol("║").set_fg(color);
        }
    }
}

/// Adds a continuous scanning sweep effect around the border perimeter
/// Creates a bright spot that travels around the border edges
fn draw_border_scanner(buf: &mut Buffer, area: Rect, time_ms: u64, color: Color) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let max_x = area.right().saturating_sub(1);
    let max_y = area.bottom().saturating_sub(1);

    // Calculate total perimeter (excluding corners to avoid double counting)
    let perimeter = ((area.width - 2) * 2 + (area.height - 2) * 2) as f64;

    // Scanner completes a full loop every 3 seconds
    let scan_duration = 3000.0;
    let scan_progress = (time_ms as f64 % scan_duration) / scan_duration;
    let current_position = (scan_progress * perimeter) as u16;

    // Track position along perimeter
    let mut pos = 0u16;

    // Helper to check if we should glow at this position (subtle)
    let should_glow = |p: u16| -> Option<Color> {
        let dist = if p > current_position {
            p.saturating_sub(current_position)
                .min(current_position + perimeter as u16 - p)
        } else {
            current_position.saturating_sub(p)
        };

        match dist {
            0 => Some(brighten_color(color)), // Scanner head (subtle)
            _ => None,                        // No glow trail
        }
    };

    // Top edge (left to right)
    for x in area.x + 1..max_x {
        if let Some(glow_color) = should_glow(pos) {
            buf[(x, area.y)].set_fg(glow_color);
        }
        pos += 1;
    }

    // Right edge (top to bottom)
    for y in area.y + 1..max_y {
        if let Some(glow_color) = should_glow(pos) {
            buf[(max_x, y)].set_fg(glow_color);
        }
        pos += 1;
    }

    // Bottom edge (right to left)
    for x in (area.x + 1..max_x).rev() {
        if let Some(glow_color) = should_glow(pos) {
            buf[(x, max_y)].set_fg(glow_color);
        }
        pos += 1;
    }

    // Left edge (bottom to top)
    for y in (area.y + 1..max_y).rev() {
        if let Some(glow_color) = should_glow(pos) {
            buf[(area.x, y)].set_fg(glow_color);
        }
        pos += 1;
    }
}

/// Brightens a color to its lighter variant (terminal-aware).
/// Shared by the progress-bar scanner and the border scanner.
pub fn brighten_color(color: Color) -> Color {
    match color {
        Color::Black => Color::DarkGray,
        Color::DarkGray => Color::Gray,
        Color::Gray => Color::White,
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan
        | Color::White => Color::White,
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(80),
            g.saturating_add(80),
            b.saturating_add(80),
        ),
        _ => color,
    }
}

/// Find the optimal focus point (character index) for a word
/// Uses a heuristic similar to Spritz speed reading
fn find_focus_point(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        10..=13 => 3,
        _ => 4,
    }
}

pub fn render_word_display(
    frame: &mut Frame,
    state: &AppState,
    opts: &RenderOptions,
    border_progress: Option<f32>,
    time_ms: u64,
) -> Rect {
    let area = frame.area();

    let word = state.current_word().unwrap_or("");
    let preview_words = state.peek_words(opts.preview_count);
    let current_word = state.current_word_index();
    let total_words = state.total_words();
    let is_paused = state.is_paused();

    let focus_idx = find_focus_point(word);
    let chars: Vec<char> = word.chars().collect();

    // Split word into before, focus, and after
    let before: String = chars.iter().take(focus_idx).collect();
    let focus = chars
        .get(focus_idx)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let after: String = chars.iter().skip(focus_idx + 1).collect();

    // Display widths (not byte lengths) so wide/multibyte chars center correctly.
    let before_width = UnicodeWidthStr::width(before.as_str());
    let focus_width = UnicodeWidthStr::width(focus.as_str());
    let after_width = UnicodeWidthStr::width(after.as_str());
    let preview_width: usize = preview_words
        .iter()
        .map(|pw| UnicodeWidthStr::width(format!(" {}", pw).as_str()))
        .sum();

    let left_width = before_width;
    let right_width = focus_width + after_width + preview_width;

    // Progress label needs horizontal room; include it when sizing the box.
    let label_prefix = if is_paused { "⏸ " } else { "▶ " };
    let progress_label = format!("{}{}/{}", label_prefix, current_word + 1, total_words);
    let progress_label_width = UnicodeWidthStr::width(progress_label.as_str());

    // Inner width: wide enough to center the focus char and fit the progress label.
    let half_width = left_width.max(right_width);
    let word_width = (half_width * 2).max(2);
    let progress_width = progress_label_width + 6;
    let inner_width = word_width.max(progress_width);

    // One column of padding on each side, inside the border. The box also
    // keeps a little extra width beyond the word so future info (e.g. near
    // the progress/page-count label) has room to sit without crowding.
    let content_width = (inner_width + 2).max(40);
    let content_height = if opts.show_progress_bar { 2 } else { 1 };

    // Focus char sits at the horizontal center of the content area.
    let padding_left = content_width / 2 - left_width;

    // Border hugs the content instead of spanning the whole terminal.
    let show_border = opts.show_border && opts.border_color.is_some();
    let box_width = if show_border {
        content_width + 2
    } else {
        content_width
    }
    .min(area.width as usize);
    let box_height = if show_border {
        content_height + 2
    } else {
        content_height
    }
    .min(area.height as usize);

    // Center the box in the available area.
    let box_rect = Rect {
        x: area.x + (area.width as usize - box_width) as u16 / 2,
        y: area.y + (area.height as usize - box_height) as u16 / 2,
        width: box_width as u16,
        height: box_height as u16,
    };

    let content_rect = if show_border {
        if let Some(base_border_color) = opts.border_color {
            if opts.enable_animations {
                // Progressive border while animating, otherwise full border + scanner
                if let Some(progress) = border_progress {
                    draw_progressive_border(frame.buffer_mut(), box_rect, progress, base_border_color);
                } else {
                    draw_progressive_border(frame.buffer_mut(), box_rect, 1.0, base_border_color);
                    draw_border_scanner(frame.buffer_mut(), box_rect, time_ms, base_border_color);
                }
            } else {
                // No animations - just draw a simple border
                draw_progressive_border(frame.buffer_mut(), box_rect, 1.0, base_border_color);
            }
        }
        Rect {
            x: box_rect.x + 1,
            y: box_rect.y + 1,
            width: box_rect.width.saturating_sub(2),
            height: box_rect.height.saturating_sub(2),
        }
    } else {
        box_rect
    };

    // Word line: left-aligned with the focus char centered via padding.
    let dim_style = Style::default().fg(Color::DarkGray);
    let mut spans = vec![
        Span::raw(" ".repeat(padding_left)),
        Span::raw(&before),
        Span::styled(&focus, Style::default().fg(opts.focus_color).bold()),
        Span::raw(&after),
    ];
    for preview_word in preview_words.iter() {
        spans.push(Span::styled(format!(" {}", preview_word), dim_style));
    }
    let line = Line::from(spans);

    let word_rect = Rect {
        x: content_rect.x,
        y: content_rect.y,
        width: content_rect.width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Left), word_rect);

    // Progress bar sits directly below the word line.
    let progress_area = Rect {
        x: content_rect.x,
        y: content_rect.y + 1,
        width: content_rect.width,
        height: 1,
    };

    if opts.show_progress_bar {
        let progress = (current_word + 1) as f64 / total_words as f64;

        // Apply pulsing effect if animations are enabled
        let fg_color = if is_paused {
            Color::Rgb(255, 165, 0) // Orange for paused
        } else if opts.enable_animations {
            // Pulsing effect for the progress bar color (0.9-1.0 intensity)
            let pulse_cycle = 1500.0; // 1.5 second cycle
            let pulse_phase =
                (time_ms as f64 % pulse_cycle) / pulse_cycle * 2.0 * std::f64::consts::PI;
            let pulse_intensity = 0.9 + (pulse_phase.sin() * 0.1);

            // Apply pulsing to config color
            match opts.progress_bar_color {
                Color::Rgb(r, g, b) => Color::Rgb(
                    (r as f64 * pulse_intensity) as u8,
                    (g as f64 * pulse_intensity) as u8,
                    (b as f64 * pulse_intensity) as u8,
                ),
                _ => opts.progress_bar_color, // Use config color as-is for named colors
            }
        } else {
            opts.progress_bar_color // No animations - use config color as-is
        };

        // Custom progress bar with transparent background (respects terminal)
        let progress_bar = LineGauge::default()
            .filled_style(Style::default().fg(fg_color).add_modifier(Modifier::BOLD))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .line_set(symbols::line::THICK)
            .ratio(progress)
            .label(progress_label);

        frame.render_widget(progress_bar, progress_area);
        progress_area
    } else {
        // Return empty area if progress bar is disabled
        Rect::default()
    }
}

/// Renders a centered help popup overlaying the current frame.
/// Uses `Clear` to wipe the popup region before drawing so animations
/// remain visible around it without bleeding into the overlay.
/// `scroll` is a raw offset from app state — it is clamped here at render
/// time because the maximum depends on `area.height`, which is only known
/// inside the draw closure.
pub fn render_help_popup(frame: &mut Frame, border_color: Color, scroll: u16, seek_step: usize) {
    let area = frame.area();

    // Popup dimensions — clamp to available terminal space
    let popup_width = 46u16.min(area.width);
    let popup_height = 14u16.min(area.height);

    let popup_x = area.x + area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.y + area.height.saturating_sub(popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Erase whatever is beneath the popup before drawing
    frame.render_widget(Clear, popup_area);

    // Separator fills the inner width (popup minus two border chars)
    let sep_width = popup_width.saturating_sub(4) as usize;

    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let header_style = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::ITALIC);
    let dim_style = Style::default().fg(Color::DarkGray);

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("  {:<14}", "Key"), header_style),
            Span::styled("Action", header_style),
        ]),
        Line::from(Span::styled(
            format!("  {}", "─".repeat(sep_width)),
            Style::default().fg(border_color),
        )),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "q / Esc"), key_style),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "Space"), key_style),
            Span::raw("Pause / Resume"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "h / ←"), key_style),
            Span::raw(format!("Rewind {} words", seek_step)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "l / →"), key_style),
            Span::raw(format!("Fast-forward {} words", seek_step)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "?"), key_style),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<14}", "j / k / ↑↓"), key_style),
            Span::raw("Scroll help"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press ? or Esc to close", dim_style)),
    ];

    // Clamp scroll so we never show empty space below the last line.
    // inner_height = popup height minus top and bottom border rows.
    let total_lines = lines.len() as u16;
    let inner_height = popup_height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(inner_height);
    let effective_scroll = scroll.min(max_scroll);

    // Build the border block, adding a scroll hint on the bottom border
    // edge only when the content actually overflows the visible area.
    let base_block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color));

    let block = if max_scroll > 0 {
        let hint = match (effective_scroll > 0, effective_scroll < max_scroll) {
            (false, true) => " ↓ j/k ",
            (true, true) => " ↑↓ j/k ",
            (true, false) => " ↑ j/k ",
            _ => "",
        };
        base_block.title_bottom(Line::from(Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        )))
    } else {
        base_block
    };

    frame.render_widget(
        Paragraph::new(lines)
            .scroll((effective_scroll, 0))
            .block(block),
        popup_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- find_focus_point ---

    #[test]
    fn focus_point_single_char() {
        assert_eq!(find_focus_point("a"), 0);
    }

    #[test]
    fn focus_point_two_to_five_chars() {
        assert_eq!(find_focus_point("ab"), 1);
        assert_eq!(find_focus_point("abc"), 1);
        assert_eq!(find_focus_point("hello"), 1); // 5 chars
    }

    #[test]
    fn focus_point_six_to_nine_chars() {
        assert_eq!(find_focus_point("foobar"), 2); // 6 chars
        assert_eq!(find_focus_point("something"), 2); // 9 chars
    }

    #[test]
    fn focus_point_ten_to_thirteen_chars() {
        assert_eq!(find_focus_point("abcdefghij"), 3); // 10 chars
        assert_eq!(find_focus_point("abcdefghijklm"), 3); // 13 chars
    }

    #[test]
    fn focus_point_fourteen_plus_chars() {
        assert_eq!(find_focus_point("abcdefghijklmn"), 4); // 14 chars
        assert_eq!(find_focus_point("abcdefghijklmnopqrstuvwxyz"), 4); // 26 chars
    }

    #[test]
    fn focus_point_counts_unicode_chars_not_bytes() {
        // "café" = 4 chars (c, a, f, é) → bucket 2..=5 → 1
        assert_eq!(find_focus_point("café"), 1);
        // "naïveté" = 7 chars → bucket 6..=9 → 2
        assert_eq!(find_focus_point("naïveté"), 2);
    }

    // --- brighten_color ---

    #[test]
    fn brighten_color_lifts_named_colors() {
        assert_eq!(brighten_color(Color::Black), Color::DarkGray);
        assert_eq!(brighten_color(Color::DarkGray), Color::Gray);
        assert_eq!(brighten_color(Color::Gray), Color::White);
        assert_eq!(brighten_color(Color::Cyan), Color::LightCyan);
    }

    #[test]
    fn brighten_color_lifts_rgb() {
        assert_eq!(
            brighten_color(Color::Rgb(60, 100, 100)),
            Color::Rgb(140, 180, 180)
        );
    }

    #[test]
    fn brighten_color_saturates_at_white() {
        assert_eq!(
            brighten_color(Color::Rgb(250, 250, 250)),
            Color::Rgb(255, 255, 255)
        );
    }

}
