use crate::state::{AppState, TokenKind};
use ratatui::{
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
    pub show_border: bool,
    pub show_progress_bar: bool,
}

/// The regions `render_word_display` painted, returned so callers can target
/// TachyonFX effects at the word, the progress bar, or the whole box.
pub struct RenderAreas {
    /// Full box including the border (or the content box when borderless).
    pub box_area: Rect,
    /// The word line.
    pub word: Rect,
    /// The progress bar row (zero-sized when disabled).
    pub progress: Rect,
}

/// Style for a token kind. The focus letter is styled separately so it stays
/// readable; this highlights Markdown structure (bold, code, links, ...).
fn kind_style(kind: TokenKind, opts: &RenderOptions) -> Style {
    match kind {
        TokenKind::Normal => Style::default(),
        TokenKind::Bold => Style::default().add_modifier(Modifier::BOLD),
        TokenKind::Italic => Style::default().add_modifier(Modifier::ITALIC),
        TokenKind::BoldItalic => {
            Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC)
        }
        TokenKind::Code => Style::default().fg(Color::LightCyan).bg(Color::DarkGray),
        TokenKind::Heading => Style::default()
            .fg(opts.focus_color)
            .add_modifier(Modifier::BOLD),
        TokenKind::Link => Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::UNDERLINED),
        TokenKind::Strikethrough => Style::default().add_modifier(Modifier::CROSSED_OUT),
        TokenKind::Blockquote => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
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
) -> RenderAreas {
    let area = frame.area();

    let token = state.current_token();
    let kind = token.map(|t| t.kind).unwrap_or(TokenKind::Normal);
    let word = token.map(|t| t.text).unwrap_or("");
    let preview_tokens = state.peek_tokens(opts.preview_count);
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
    let preview_width: usize = preview_tokens
        .iter()
        .map(|pt| UnicodeWidthStr::width(format!(" {}", pt.text).as_str()))
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

    // Draw the border around the box; TachyonFX reveals it on startup.
    if show_border {
        if let Some(base_border_color) = opts.border_color {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(base_border_color));
            frame.render_widget(block, box_rect);
        }
    }

    let content_rect = if show_border {
        Rect {
            x: box_rect.x + 1,
            y: box_rect.y + 1,
            width: box_rect.width.saturating_sub(2),
            height: box_rect.height.saturating_sub(2),
        }
    } else {
        box_rect
    };

    // Center the focus char in the *visible* content area. When a word is
    // wider than the box (clamped to the terminal), this keeps the focus
    // letter on-screen instead of pushing the whole line off to the right.
    let padding_left = (content_rect.width as usize / 2).saturating_sub(left_width);

    // Word line: left-aligned with the focus char centered via padding.
    // Markdown structure is styled by token kind; the focus letter keeps its
    // own color so it stays prominent regardless of the surrounding syntax.
    let word_style = kind_style(kind, opts);
    let mut spans = vec![
        Span::raw(" ".repeat(padding_left)),
        Span::styled(before, word_style),
        Span::styled(&focus, Style::default().fg(opts.focus_color).bold()),
        Span::styled(after, word_style),
    ];
    for pt in preview_tokens.iter() {
        let style = kind_style(pt.kind, opts).fg(Color::DarkGray);
        spans.push(Span::styled(format!(" {}", pt.text), style));
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
    let progress_rect = Rect {
        x: content_rect.x,
        y: content_rect.y + 1,
        width: content_rect.width,
        height: 1,
    };

    if opts.show_progress_bar {
        let progress = (current_word + 1) as f64 / total_words as f64;
        let fg_color = if is_paused {
            Color::Rgb(255, 165, 0) // Orange for paused
        } else {
            opts.progress_bar_color
        };

        // Custom progress bar with transparent background (respects terminal)
        let progress_bar = LineGauge::default()
            .filled_style(Style::default().fg(fg_color).add_modifier(Modifier::BOLD))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .line_set(symbols::line::THICK)
            .ratio(progress)
            .label(progress_label);

        frame.render_widget(progress_bar, progress_rect);
    }

    RenderAreas {
        box_area: box_rect,
        word: word_rect,
        progress: progress_rect,
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

    // --- render_word_display ---

    #[test]
    fn render_word_display_draws_border_word_and_progress() {
        use ratatui::backend::TestBackend;

        let state = AppState::new("hello world", 300);
        let opts = RenderOptions {
            preview_count: 0,
            border_color: Some(Color::Cyan),
            progress_bar_color: Color::Cyan,
            focus_color: Color::Red,
            show_border: true,
            show_progress_bar: true,
        };

        let backend = TestBackend::new(50, 10);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut areas: Option<RenderAreas> = None;
        terminal
            .draw(|f| {
                areas = Some(render_word_display(f, &state, &opts));
            })
            .unwrap();

        let areas = areas.unwrap();
        let buf = terminal.backend().buffer();

        let mut text = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                text.push_str(buf[(x, y)].symbol());
            }
        }

        assert!(text.contains("hello"), "word should be rendered");
        assert!(text.contains('╔') || text.contains('═'), "border should be rendered");
        assert!(areas.word.width > 0);
        assert!(areas.progress.width > 0);
        assert!(areas.box_area.width >= areas.word.width);
    }
}

