use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    text::{Line, Span},
    widgets::Paragraph,
};

pub struct UIConstraints {
    pub constraints: [Constraint; 3],
}

impl UIConstraints {
    pub fn new(is_inline: bool) -> Self {
        let constraints = if !is_inline {
            [
                Constraint::Percentage(50),
                Constraint::Min(1),
                Constraint::Percentage(50),
            ]
        } else {
            [
                Constraint::Percentage(10),
                Constraint::Min(1),
                Constraint::Percentage(10),
            ]
        };

        Self { constraints }
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

pub fn render_word_display(frame: &mut Frame, word: &str, constraints: &UIConstraints) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.constraints)
        .split(frame.area());

    let focus_idx = find_focus_point(word);
    let chars: Vec<char> = word.chars().collect();

    // Split word into before, focus, and after
    let before: String = chars.iter().take(focus_idx).collect();
    let focus = chars.get(focus_idx).map(|c| c.to_string()).unwrap_or_default();
    let after: String = chars.iter().skip(focus_idx + 1).collect();

    // Calculate padding to center the focus character
    let term_width = chunks[1].width as usize;
    let focus_position = term_width / 2;
    let padding_left = if focus_position > before.len() {
        focus_position - before.len()
    } else {
        0
    };

    // Build the line with styled spans
    let line = Line::from(vec![
        Span::raw(" ".repeat(padding_left)),
        Span::raw(&before),
        Span::styled(&focus, Style::default().fg(Color::Red).bold()),
        Span::raw(&after),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Left);
    frame.render_widget(paragraph, chunks[1]);
}