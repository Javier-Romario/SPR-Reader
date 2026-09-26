use std::time::{Duration, Instant};

/// Tokens longer than this are split further so a single flash stays readable.
const LONG_TOKEN_CHARS: usize = 24;

/// How a token should be rendered, derived from its Markdown context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Heading,
    Link,
    Strikethrough,
    Blockquote,
}

/// A single reading unit: the visible text plus its Markdown-derived kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
}

/// Split Markdown content into reading units, stripping the Markdown syntax and
/// tagging each token with its kind so the reader can highlight it correctly.
///
/// Handles headings, blockquotes, lists, fenced code blocks, horizontal rules,
/// inline code, bold/italic (self-contained and spanning), strikethrough, and
/// links.
fn tokenize(content: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut in_fence = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Toggle fenced code blocks (``` or ~~~), skipping the fence lines.
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }

        if in_fence {
            for token in line.split_whitespace() {
                out.push(Token { text: token, kind: TokenKind::Code });
            }
            continue;
        }

        // Blockquote.
        let (body, base) = if let Some(rest) = trimmed.strip_prefix('>') {
            (rest.strip_prefix(' ').unwrap_or(rest), TokenKind::Blockquote)
        } else {
            (line, TokenKind::Normal)
        };

        // Heading (hashes followed by a space, per CommonMark).
        let (body, base) = match strip_heading(body) {
            Some(rest) => (rest, TokenKind::Heading),
            None => (body, base),
        };

        // Horizontal rule.
        if is_horizontal_rule(body) {
            continue;
        }

        // List marker (-, *, +, N., N)) is dropped; the text reads normally.
        let body = strip_list_marker(body).unwrap_or(body);

        push_inline(body, base, &mut out);
    }

    out
}

fn strip_heading(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    rest.strip_prefix(' ')
}

fn is_horizontal_rule(line: &str) -> bool {
    let t = line.trim();
    if t.len() < 3 {
        return false;
    }
    let c = t.as_bytes()[0];
    (c == b'-' || c == b'*' || c == b'_') && t.bytes().all(|b| b == c)
}

fn strip_list_marker(line: &str) -> Option<&str> {
    let t = line.trim_start();
    if let Some(rest) = t
        .strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .or_else(|| t.strip_prefix("+ "))
    {
        return Some(rest);
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let rest = &t[digits..];
        if let Some(rest) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
            return Some(rest);
        }
    }
    None
}

fn push_inline<'a>(line: &'a str, base: TokenKind, out: &mut Vec<Token<'a>>) {
    let mut bold = false;
    let mut italic = false;
    let mut code = false;
    let mut strike = false;

    for raw in line.split_whitespace() {
        let (text, kind) = classify_inline(raw, &mut bold, &mut italic, &mut code, &mut strike);
        if text.is_empty() {
            continue;
        }
        let kind = if kind == TokenKind::Normal { base } else { kind };
        if text.chars().count() > LONG_TOKEN_CHARS {
            split_long_token(text, kind, out);
        } else {
            out.push(Token { text, kind });
        }
    }
}

fn classify_inline<'a>(
    raw: &'a str,
    bold: &mut bool,
    italic: &mut bool,
    code: &mut bool,
    strike: &mut bool,
) -> (&'a str, TokenKind) {
    // Inline code (backticks), including spanning across tokens.
    if *code {
        if raw.ends_with('`') {
            *code = false;
            return (raw.trim_end_matches('`'), TokenKind::Code);
        }
        return (raw, TokenKind::Code);
    }
    if raw.starts_with('`') {
        if raw.len() >= 2 && raw.ends_with('`') {
            return (raw.trim_matches('`'), TokenKind::Code);
        }
        *code = true;
        return (raw.trim_start_matches('`'), TokenKind::Code);
    }

    // Strikethrough (~~).
    if raw.starts_with("~~") && raw.ends_with("~~") && raw.len() >= 4 {
        return (
            raw.trim_start_matches('~').trim_end_matches('~'),
            TokenKind::Strikethrough,
        );
    }
    if raw.starts_with("~~") {
        *strike = true;
        return (raw.trim_start_matches('~'), TokenKind::Strikethrough);
    }
    if raw.ends_with("~~") {
        *strike = false;
        return (raw.trim_end_matches('~'), TokenKind::Strikethrough);
    }
    if *strike {
        return (raw, TokenKind::Strikethrough);
    }

    // Links and images: [text](url), ![alt](url).
    if let Some(rest) = raw.strip_prefix('!') {
        if let Some(text) = link_text(rest) {
            return (text, TokenKind::Link);
        }
    }
    if let Some(text) = link_text(raw) {
        return (text, TokenKind::Link);
    }

    // Emphasis (* and _): bold, italic, or bold-italic.
    let lead = raw.len() - raw.trim_start_matches(['*', '_']).len();
    let trail = raw.len() - raw.trim_end_matches(['*', '_']).len();
    let text = if lead + trail <= raw.len() {
        &raw[lead..raw.len() - trail]
    } else {
        ""
    };

    let kind = if lead >= 3 && trail >= 3 {
        TokenKind::BoldItalic
    } else if lead >= 2 && trail >= 2 {
        TokenKind::Bold
    } else if lead >= 1 && trail >= 1 {
        TokenKind::Italic
    } else if lead >= 2 {
        *bold = true;
        TokenKind::Bold
    } else if trail >= 2 {
        *bold = false;
        TokenKind::Bold
    } else if lead >= 1 {
        *italic = true;
        TokenKind::Italic
    } else if trail >= 1 {
        *italic = false;
        TokenKind::Italic
    } else if *bold && *italic {
        TokenKind::BoldItalic
    } else if *bold {
        TokenKind::Bold
    } else if *italic {
        TokenKind::Italic
    } else {
        TokenKind::Normal
    };

    (text, kind)
}

fn link_text(raw: &str) -> Option<&str> {
    let open = raw.find('[')?;
    let close = raw[open + 1..].find(']')? + open + 1;
    let text = &raw[open + 1..close];
    if text.is_empty() {
        return None;
    }
    Some(text)
}

/// Split a long token on punctuation and camelCase boundaries, dropping the
/// separator characters themselves and keeping the token's kind.
fn split_long_token<'a>(token: &'a str, kind: TokenKind, out: &mut Vec<Token<'a>>) {
    let mut byte_start = 0;
    let mut prev_lower_or_digit = false;

    for (bi, c) in token.char_indices() {
        let is_punct = matches!(c, '/' | '\\' | '_' | '-' | '.' | ':' | '|' | '`');
        let is_camel = c.is_uppercase() && prev_lower_or_digit;

        if is_punct || is_camel {
            if byte_start < bi {
                out.push(Token { text: &token[byte_start..bi], kind });
            }
            byte_start = bi + if is_punct { c.len_utf8() } else { 0 };
        }

        prev_lower_or_digit = c.is_lowercase() || c.is_numeric();
    }

    if byte_start < token.len() {
        out.push(Token { text: &token[byte_start..], kind });
    }
}

#[derive(Debug)]
pub struct AppState<'a> {
    tokens: Vec<Token<'a>>,
    current_word: usize,
    paused: bool,
    wpm: u64,
    next_tick: Instant,
}

impl<'a> AppState<'a> {
    pub fn new(content: &'a str, wpm: u64) -> Self {
        let tokens = tokenize(content);
        let mut state = Self {
            tokens,
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
        self.tokens.get(self.current_word).map(|t| t.text)
    }

    /// The current token (text + kind) for styling.
    pub fn current_token(&self) -> Option<&Token<'_>> {
        self.tokens.get(self.current_word)
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn should_advance(&self) -> bool {
        Instant::now() >= self.next_tick && !self.paused
    }

    pub fn advance_word(&mut self) -> bool {
        self.current_word += 1;
        if self.current_word >= self.tokens.len() {
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
        self.tokens.len()
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Upcoming tokens (text + kind) for preview styling.
    pub fn peek_tokens(&self, count: usize) -> Vec<&Token<'_>> {
        let start = self.current_word + 1;
        self.tokens[start..].iter().take(count).collect()
    }

    /// Jump forward or backward by `delta` words (clamped to word bounds).
    /// Resets the tick timer so the landed-on word gets a full display window.
    pub fn seek_word(&mut self, delta: isize) {
        let new_index = (self.current_word as isize + delta)
            .max(0)
            .min((self.tokens.len() as isize).saturating_sub(1)) as usize;
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

    // --- tokenize ---

    fn t(text: &str, kind: TokenKind) -> Token<'_> {
        Token { text, kind }
    }

    use TokenKind::{Bold, BoldItalic, Blockquote, Code, Heading, Italic, Link, Normal, Strikethrough};

    #[test]
    fn tokenize_keeps_short_tokens_whole() {
        assert_eq!(
            tokenize("iPhone well-known"),
            vec![t("iPhone", Normal), t("well-known", Normal)]
        );
    }

    #[test]
    fn tokenize_inline_code_backticks() {
        assert_eq!(
            tokenize("the `usePipSdkBootstrap` hook"),
            vec![
                t("the", Normal),
                t("usePipSdkBootstrap", Code),
                t("hook", Normal),
            ]
        );
    }

    #[test]
    fn tokenize_splits_long_code_identifiers_on_camel_and_punct() {
        let content = "`usePipSdkBootstrap`/`useReviewActions`/`useDraft`";
        assert_eq!(
            tokenize(content),
            vec![
                t("use", Code),
                t("Pip", Code),
                t("Sdk", Code),
                t("Bootstrap", Code),
                t("use", Code),
                t("Review", Code),
                t("Actions", Code),
                t("use", Code),
                t("Draft", Code),
            ]
        );
    }

    #[test]
    fn tokenize_splits_long_paths_on_slashes() {
        let content = "src/ratatui/tachyonfx/effect-showcase/src/main.rs";
        assert_eq!(
            tokenize(content),
            vec![
                t("src", Normal),
                t("ratatui", Normal),
                t("tachyonfx", Normal),
                t("effect", Normal),
                t("showcase", Normal),
                t("src", Normal),
                t("main", Normal),
                t("rs", Normal),
            ]
        );
    }

    #[test]
    fn tokenize_emphasis() {
        assert_eq!(
            tokenize("**bold** and *italic* and ***both***"),
            vec![
                t("bold", Bold),
                t("and", Normal),
                t("italic", Italic),
                t("and", Normal),
                t("both", BoldItalic),
            ]
        );
    }

    #[test]
    fn tokenize_spanning_bold() {
        assert_eq!(
            tokenize("**bold phrase** here"),
            vec![
                t("bold", Bold),
                t("phrase", Bold),
                t("here", Normal),
            ]
        );
    }

    #[test]
    fn tokenize_link() {
        assert_eq!(
            tokenize("see [docs](https://example.com) now"),
            vec![
                t("see", Normal),
                t("docs", Link),
                t("now", Normal),
            ]
        );
    }

    #[test]
    fn tokenize_heading() {
        assert_eq!(
            tokenize("# Title here"),
            vec![t("Title", Heading), t("here", Heading)]
        );
    }

    #[test]
    fn tokenize_blockquote() {
        assert_eq!(
            tokenize("> quoted text"),
            vec![t("quoted", Blockquote), t("text", Blockquote)]
        );
    }

    #[test]
    fn tokenize_code_fence() {
        let md = "```\nfn main() {}\n```";
        assert_eq!(
            tokenize(md),
            vec![
                t("fn", Code),
                t("main()", Code),
                t("{}", Code),
            ]
        );
    }

    #[test]
    fn tokenize_strikethrough_and_list() {
        assert_eq!(
            tokenize("- ~~done~~ and *more*"),
            vec![
                t("done", Strikethrough),
                t("and", Normal),
                t("more", Italic),
            ]
        );
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

    // --- peek_tokens ---

    #[test]
    fn peek_tokens_returns_upcoming() {
        let state = make_state("one two three four");
        let got: Vec<&str> = state.peek_tokens(2).iter().map(|t| t.text).collect();
        assert_eq!(got, vec!["two", "three"]);
    }

    #[test]
    fn peek_tokens_zero_count_is_empty() {
        assert!(make_state("one two three").peek_tokens(0).is_empty());
    }

    #[test]
    fn peek_tokens_count_exceeds_remaining() {
        let state = make_state("one two three");
        let got: Vec<&str> = state.peek_tokens(10).iter().map(|t| t.text).collect();
        assert_eq!(got, vec!["two", "three"]);
    }

    #[test]
    fn peek_tokens_at_last_word_is_empty() {
        let mut state = make_state("one two");
        state.seek_word(1); // at "two"
        assert!(state.peek_tokens(3).is_empty());
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
