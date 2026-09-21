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
        let mut state = Self {
            words,
            current_word: 0,
            paused: false,
            wpm,
            next_tick: Instant::now(),
        };
        state.next_tick = Instant::now() + state.current_word_delay();
        state
    }

    /// Time the current word should stay on screen: base WPM interval plus an
    /// extra pause when the word ends a sentence.
    ///
    /// The pause is folded into `next_tick` (not the event-poll timeout), so it
    /// genuinely slows the reader instead of being a no-op.
    fn current_word_delay(&self) -> Duration {
        let base = Duration::from_secs_f64(60.0 / self.wpm as f64);
        let ends_sentence = self
            .current_word()
            .and_then(|word| word.chars().last())
            .is_some_and(|c| matches!(c, '.' | '!' | '?' | ';'));
        if ends_sentence {
            base + Duration::from_millis(500)
        } else {
            base
        }
    }

    pub fn current_word(&self) -> Option<&str> {
        self.words.get(self.current_word).copied()
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
        self.next_tick = Instant::now() + self.current_word_delay();
        true // More words remaining
    }

    pub fn get_timeout(&self) -> Duration {
        if self.paused {
            // Block briefly while paused; key events still wake the poll
            // immediately, so responsiveness is unaffected. Avoids a busy loop
            // once `next_tick` has elapsed.
            return Duration::from_millis(100);
        }
        self.next_tick.saturating_duration_since(Instant::now())
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

    /// Returns up to `count` words following the current word.
    pub fn peek_words(&self, count: usize) -> Vec<&str> {
        let start = self.current_word + 1;
        self.words[start..].iter().take(count).copied().collect()
    }

    /// Jump forward or backward by `delta` words (clamped to word bounds).
    /// Resets the tick timer so the landed-on word gets a full display window.
    pub fn seek_word(&mut self, delta: isize) {
        let new_index = (self.current_word as isize + delta)
            .max(0)
            .min((self.words.len() as isize).saturating_sub(1)) as usize;
        self.current_word = new_index;
        self.next_tick = Instant::now() + self.current_word_delay();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(content: &str) -> AppState<'_> {
        AppState::new(content, 300)
    }

    // --- construction ---

    #[test]
    fn new_splits_by_whitespace() {
        assert_eq!(make_state("one two three").total_words(), 3);
    }

    #[test]
    fn new_collapses_extra_whitespace() {
        assert_eq!(make_state("  one   two  ").total_words(), 2);
    }

    #[test]
    fn new_single_word() {
        assert_eq!(make_state("only").total_words(), 1);
    }

    // --- current_word ---

    #[test]
    fn current_word_starts_at_first() {
        assert_eq!(make_state("hello world").current_word(), Some("hello"));
    }

    #[test]
    fn current_word_index_starts_at_zero() {
        assert_eq!(make_state("a b c").current_word_index(), 0);
    }

    // --- advance_word ---

    #[test]
    fn advance_word_moves_to_next() {
        let mut state = make_state("one two three");
        assert!(state.advance_word());
        assert_eq!(state.current_word(), Some("two"));
    }

    #[test]
    fn advance_word_returns_false_when_past_last() {
        let mut state = make_state("only");
        assert!(!state.advance_word());
    }

    #[test]
    fn advance_word_through_all_words() {
        let mut state = make_state("a b c");
        assert!(state.advance_word()); // -> b
        assert!(state.advance_word()); // -> c
        assert!(!state.advance_word()); // past end
    }

    #[test]
    fn advance_word_updates_index() {
        let mut state = make_state("a b c");
        state.advance_word();
        assert_eq!(state.current_word_index(), 1);
        state.advance_word();
        assert_eq!(state.current_word_index(), 2);
    }

    // --- toggle_pause ---

    #[test]
    fn starts_unpaused() {
        assert!(!make_state("hello").is_paused());
    }

    #[test]
    fn toggle_pause_toggles() {
        let mut state = make_state("hello");
        state.toggle_pause();
        assert!(state.is_paused());
        state.toggle_pause();
        assert!(!state.is_paused());
    }

    #[test]
    fn should_advance_is_false_when_paused() {
        let mut state = make_state("hello world");
        state.toggle_pause();
        assert!(!state.should_advance());
    }

    // --- seek_word ---

    #[test]
    fn seek_forward() {
        let mut state = make_state("a b c d e");
        state.seek_word(3);
        assert_eq!(state.current_word_index(), 3);
        assert_eq!(state.current_word(), Some("d"));
    }

    #[test]
    fn seek_backward() {
        let mut state = make_state("a b c d e");
        state.seek_word(4); // -> e
        state.seek_word(-2); // -> c
        assert_eq!(state.current_word_index(), 2);
        assert_eq!(state.current_word(), Some("c"));
    }

    #[test]
    fn seek_clamps_at_start() {
        let mut state = make_state("a b c");
        state.seek_word(-100);
        assert_eq!(state.current_word_index(), 0);
        assert_eq!(state.current_word(), Some("a"));
    }

    #[test]
    fn seek_clamps_at_end() {
        let mut state = make_state("a b c");
        state.seek_word(100);
        assert_eq!(state.current_word_index(), 2);
        assert_eq!(state.current_word(), Some("c"));
    }

    #[test]
    fn seek_zero_delta_is_noop() {
        let mut state = make_state("a b c");
        state.seek_word(1);
        state.seek_word(0);
        assert_eq!(state.current_word_index(), 1);
    }

    // --- peek_words ---

    #[test]
    fn peek_words_returns_upcoming() {
        let state = make_state("one two three four");
        assert_eq!(state.peek_words(2), vec!["two", "three"]);
    }

    #[test]
    fn peek_words_zero_count_is_empty() {
        assert!(make_state("one two three").peek_words(0).is_empty());
    }

    #[test]
    fn peek_words_count_exceeds_remaining() {
        let state = make_state("one two three");
        assert_eq!(state.peek_words(10), vec!["two", "three"]);
    }

    #[test]
    fn peek_words_at_last_word_is_empty() {
        let mut state = make_state("one two");
        state.seek_word(1); // at "two"
        assert!(state.peek_words(3).is_empty());
    }

    // --- wpm / timing ---

    #[test]
    fn wpm_300_delay_is_200ms() {
        // 60 / 300 = 0.2 seconds = 200ms
        let state = AppState::new("hello", 300);
        let timeout = state.get_timeout();
        // timeout should be ~200ms (allow ±50ms for test overhead)
        assert!(timeout.as_millis() <= 250);
    }

    #[test]
    fn sentence_ending_punctuation_adds_delay() {
        // get_timeout adds 500ms for sentence-ending punctuation
        let state = AppState::new("Hello.", 300);
        let timeout = state.get_timeout();
        // Base ~200ms + 500ms punctuation = ~700ms
        assert!(timeout.as_millis() >= 450);
    }

    #[test]
    fn non_sentence_punctuation_no_extra_delay() {
        let state = AppState::new("hello,", 300);
        let timeout = state.get_timeout();
        assert!(timeout.as_millis() <= 250);
    }

    #[test]
    fn get_timeout_blocks_briefly_while_paused() {
        let mut state = AppState::new("hello world", 300);
        state.toggle_pause();
        assert_eq!(state.get_timeout(), Duration::from_millis(100));
    }

    #[test]
    fn advance_onto_sentence_punctuation_adds_delay() {
        let mut state = AppState::new("go stop.", 300);
        state.advance_word(); // -> "stop." ends with '.'
        assert!(state.get_timeout().as_millis() >= 450);
    }

    #[test]
    fn advance_onto_non_sentence_punctuation_keeps_base_delay() {
        let mut state = AppState::new("go stop,", 300);
        state.advance_word(); // -> "stop," no extra delay
        assert!(state.get_timeout().as_millis() <= 250);
    }
}
