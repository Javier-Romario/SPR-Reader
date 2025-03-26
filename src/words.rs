use ratatui::{
    prelude::Line,
    text::Span,
    prelude::Alignment,
    style::{
        Style,
        Color,
    }
};

pub fn process_words(text: &str) -> Line {
    // Display current word
    // let paragraph = Paragraph::new(words[current_word])
    //     .alignment(Alignment::Center);
    // f.render_widget(paragraph, chunks[1]);
    let processed = text.chars().count();
    if processed == 0 {
        return Line::from("");
    }
    if processed == 1 {
        return Line::from(text).alignment(Alignment::Center).style(Style::default().fg(Color::Cyan));
    }
    if processed == 2 {
        return Line::from(
            vec![
                Span::from(&text[0..1]),
                Span::from(&text[1..]).style(Style::default().fg(Color::Cyan)),
            ]
        ).alignment(Alignment::Center);
    }
    if processed == 3 {
        return Line::from(
            vec![
                Span::from(&text[0..1]),
                Span::from(&text[1..2]).style(Style::default().fg(Color::Cyan)),
                Span::from(&text[2..]),
            ]
        ).alignment(Alignment::Center);
    }
    else {
        let split_word = text;
        let split_word = split_word.split_at(text.chars().count() / 2);

        // if word is > 3
        let first = Span::raw(split_word.0);
        let mid = if (split_word.1.chars().count() / 2) % 2 == 0 {
            &text.chars().count() / 2
        } else {
            &text.chars().count() / 2 - 1
        };


        let second_split = split_word.1.split_at(mid);
        let second = Span::styled(second_split.0, Style::default().fg(Color::Cyan));
        let third = Span::raw(second_split.1);
        Line::from(vec![first, second, third]).alignment(Alignment::Center)
    }
}
