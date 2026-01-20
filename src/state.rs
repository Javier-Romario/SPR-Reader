use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct AppState<'a> {
    words: Vec<&'a str>,
    current_word: usize,
    paused: bool,
    wpm: u64,
    next_tick: Instant,
}

impl<'a> AppState<'a> {
    pub fn new(content: &'a str, wpm: u64) -> Self {
        let words: Vec<&'a str> = content.split_whitespace().collect();
        let delay = Duration::from_secs_f64(60.0 / wpm as f64);

        Self {
            words,
            current_word: 0,
            paused: false,
            wpm,
            next_tick: Instant::now() + delay,
        }
    }

    pub fn current_word(&self) -> &str {
        self.words[self.current_word]
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn should_advance(&self) -> bool {
        Instant::now() >= self.next_tick && !self.paused
    }

    pub fn advance_word(&mut self) -> bool {
        self.current_word += 1;
        if self.current_word >= self.words.len() {
            return false; // No more words
        }
        let delay = Duration::from_secs_f64(60.0 / self.wpm as f64);
        self.next_tick = Instant::now() + delay;
        true // More words remaining
    }

    pub fn get_timeout(&self) -> Duration {
        let last_char = self.current_word().chars().last().unwrap_or(' ');

        // Only pause on sentence-ending punctuation
        let punctuation_delay = if matches!(last_char, '.' | '!' | '?' | ';') {
            Duration::from_secs_f64(0.5)
        } else {
            Duration::from_secs(0)
        };

        self.next_tick.saturating_duration_since(Instant::now()) + punctuation_delay
    }

    pub fn current_word_index(&self) -> usize {
        self.current_word
    }

    pub fn total_words(&self) -> usize {
        self.words.len()
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
