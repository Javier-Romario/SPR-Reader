use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::Paragraph,
    TerminalOptions,
    Viewport
};
use std::{
    fs,
    io::{self, stdout},
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Text to read
    #[arg(short, long, conflicts_with = "file")]
    text: Option<String>,

    /// File to read text from
    #[arg(short, long)]
    file: Option<String>,

    /// Words per minute
    #[arg(long, default_value = "300")]
    wpm: u64,

    /// Inline mode
    #[arg(short, long, default_value = "false")]
    inline: Option<bool>,
}

struct Mode {
    paused: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Read input text
    let content = match (&args.file, &args.text, &args.inline) {
        // file
        (Some(file), None, None) => fs::read_to_string(file)?,
        (Some(file), None, Some(true)) => fs::read_to_string(file)?,
        (Some(file), None, Some(false)) => fs::read_to_string(file)?,
        // text
        (None, Some(text), None) => text.clone(),
        (None, Some(text), Some(true)) => text.clone(),
        (None, Some(text), Some(false)) => text.clone(),
        _ => return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Either --text or --file must be provided",
        ).into()),
    };

    let mut state = Mode { paused: false };

    // Split text into words
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No words to display",
        ).into());
    }

    let is_inline = args.inline.unwrap_or(false);

    // Calculate delay between words
    let delay = Duration::from_secs_f64(60.0 / args.wpm as f64);

    if is_inline == false {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
    }

    let mut terminal = ratatui::init_with_options(TerminalOptions {
        viewport: if is_inline == false {
            Viewport::Fullscreen
        } else {
            Viewport::Inline(5)
        },
    });

    let mut current_word = 0;
    let mut next_tick = Instant::now() + delay;

    // Main application loop
    let constraints = if is_inline == false {
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
            // Create centered layout
            let chunks = if is_inline == false {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(f.area())
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(f.area())
            };

            // Display current word
            let paragraph = Paragraph::new(words[current_word])
                .alignment(Alignment::Center);
            f.render_widget(paragraph, chunks[1]);
        })?;

        // Calculate time until next word update
        let timeout = next_tick.saturating_duration_since(Instant::now());

        // Check for user input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => state.paused = !state.paused,
                    _ => {}
                }
            }
        }

        // Update word if time elapsed and not paused
        if Instant::now() >= next_tick && state.paused == false {
            current_word = (current_word + 1) % words.len();
            next_tick = Instant::now() + delay;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
