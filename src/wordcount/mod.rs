//! Word count + reading-time estimate for markdown / plain text.
//!
//! Used by the status bar; intentionally cheap — a single pass over the
//! buffer counting runs of word characters.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordStats {
    pub words: usize,
    pub chars: usize,
    /// Estimated reading time in **minutes**, rounded up. Caps at 1 min
    /// for any non-empty buffer.
    pub minutes: usize,
}

/// Average prose reading speed (English). Picks the middle of the
/// commonly cited 200-250 wpm range.
pub const WORDS_PER_MINUTE: usize = 225;

pub fn analyze(text: &str) -> WordStats {
    let chars = text.chars().count();
    let mut words = 0usize;
    let mut in_word = false;
    for ch in text.chars() {
        let is_word = ch.is_alphanumeric() || ch == '_' || ch == '\'' || ch == '-';
        if is_word && !in_word {
            words += 1;
            in_word = true;
        } else if !is_word {
            in_word = false;
        }
    }
    let minutes = if words == 0 {
        0
    } else {
        ((words + WORDS_PER_MINUTE - 1) / WORDS_PER_MINUTE).max(1)
    };
    WordStats { words, chars, minutes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_has_zero_words_and_minutes() {
        let s = analyze("");
        assert_eq!(s.words, 0);
        assert_eq!(s.chars, 0);
        assert_eq!(s.minutes, 0);
    }

    #[test]
    fn single_word_counts_one() {
        let s = analyze("hello");
        assert_eq!(s.words, 1);
        assert_eq!(s.chars, 5);
        assert_eq!(s.minutes, 1);
    }

    #[test]
    fn punctuation_separates_words() {
        let s = analyze("hello, world! foo.bar");
        assert_eq!(s.words, 4);
    }

    #[test]
    fn newlines_separate_words() {
        let s = analyze("one\ntwo\nthree");
        assert_eq!(s.words, 3);
    }

    #[test]
    fn hyphenated_words_count_as_one() {
        let s = analyze("state-of-the-art tool");
        assert_eq!(s.words, 2);
    }

    #[test]
    fn apostrophes_keep_word_intact() {
        let s = analyze("it's don't");
        assert_eq!(s.words, 2);
    }

    #[test]
    fn unicode_words_count() {
        let s = analyze("café résumé naïve");
        assert_eq!(s.words, 3);
    }

    #[test]
    fn minutes_rounds_up_above_threshold() {
        // 1 word ⇒ 1 minute (minimum)
        assert_eq!(analyze("x").minutes, 1);
        // Just over 225 ⇒ 2 minutes.
        let text: String = std::iter::repeat("word ").take(226).collect();
        assert_eq!(analyze(&text).minutes, 2);
    }

    #[test]
    fn whitespace_only_text_has_no_words() {
        assert_eq!(analyze("   \n  \t  \n").words, 0);
    }
}
