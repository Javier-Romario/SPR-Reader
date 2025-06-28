use crate::tui::Tui;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    widgets::Paragraph,
};
use std::{
    io,
    time::{Duration, Instant},
};

struct AppState<'a> {
    words: Vec<&'a str>,
    current_word: usize,
    paused: bool,
}

pub fn run<'a>(content: &'a str, wpm: u64, is_inline: bool, terminal: &mut Tui) -> Result<()> {
    let words: Vec<&'a str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No words to display").into());
    }

    let mut app_state = AppState {
        words,
        current_word: 0,
        paused: false,
    };

    let delay = Duration::from_secs_f64(60.0 / wpm as f64);
    let mut next_tick = Instant::now() + delay;

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

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.size());

            let paragraph = Paragraph::new(app_state.words[app_state.current_word])
                .alignment(Alignment::Center);
            f.render_widget(paragraph, chunks[1]);
        })?;

        let punctuation_delay = if app_state.words[app_state.current_word]
            .chars()
            .last()
            .unwrap()
            .is_ascii_punctuation()
        {
            Duration::from_secs_f64(0.5)
        } else {
            Duration::from_secs(0)
        };

        let timeout = next_tick.saturating_duration_since(Instant::now() - punctuation_delay);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app_state.paused = !app_state.paused,
                    _ => {}
                }
            }
        }

        if Instant::now() >= next_tick && !app_state.paused {
            app_state.current_word = (app_state.current_word + 1) % app_state.words.len();
            next_tick = Instant::now() + delay;
        }
    }

    Ok(())
}
