use clap::Parser;
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
    Terminal,
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
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Read input text
    let content = match (&args.file, &args.text) {
        (Some(file), None) => fs::read_to_string(file)?,
        (None, Some(text)) => text.clone(),
        _ => return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Either --text or --file must be provided",
        )),
    };

    // Split text into words
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No words to display",
        ));
    }

    // Calculate delay between words
    let delay = Duration::from_secs_f64(60.0 / args.wpm as f64);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_word = 0;
    let mut next_tick = Instant::now() + delay;

    // Main application loop
    loop {
        terminal.draw(|f| {
            // Create centered layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Min(1),
                    Constraint::Percentage(50),
                ])
                .split(f.size());

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
                    _ => {}
                }
            }
        }

        // Update word if time elapsed
        if Instant::now() >= next_tick {
            current_word = (current_word + 1) % words.len();
            next_tick = Instant::now() + delay;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
