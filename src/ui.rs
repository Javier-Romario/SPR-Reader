use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
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

pub fn render_word_display(frame: &mut Frame, word: &str, constraints: &UIConstraints) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.constraints)
        .split(frame.area());

    let paragraph = Paragraph::new(word).alignment(Alignment::Center);
    frame.render_widget(paragraph, chunks[1]);
}