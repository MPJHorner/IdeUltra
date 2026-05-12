//! Pure helpers for the "Select Next Occurrence" command (⌘D).
//!
//! Two functions, both unit-tested:
//!   * `word_at(text, cursor_char)` — return the char range of the word
//!     under the cursor, or `None` if the cursor isn't on a word.
//!   * `find_next(text, start_char, needle)` — return the next char
//!     range of `needle` at or after `start_char`. Wraps to the start
//!     if nothing's found in the tail.

use std::ops::Range;

#[inline]
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Return the char-index range of the word containing `cursor_char`.
/// If the cursor sits between two non-word characters, return None.
/// The cursor is allowed to sit just past the last word char (so
/// putting your caret right after `foo` still picks `foo`).
pub fn word_at(text: &str, cursor_char: usize) -> Option<Range<usize>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let n = chars.len();
    let pivot = cursor_char.min(n);

    // Determine the index of a word-char near the cursor.
    let mut idx = if pivot < n && is_word_char(chars[pivot]) {
        pivot
    } else if pivot > 0 && is_word_char(chars[pivot - 1]) {
        pivot - 1
    } else {
        return None;
    };

    // Walk left to find the start of the word.
    while idx > 0 && is_word_char(chars[idx - 1]) {
        idx -= 1;
    }
    let start = idx;
    // Walk right to find the end.
    let mut end = start;
    while end < n && is_word_char(chars[end]) {
        end += 1;
    }
    Some(start..end)
}

/// Find the next occurrence of `needle` in `text` starting at
/// char-index `start_char`. If none is found in the tail, wraps and
/// searches from the start (so the user gets cyclic behaviour).
///
/// `case_sensitive` controls matching. Pure ASCII or Unicode is fine
/// because we work in chars throughout.
pub fn find_next(
    text: &str,
    start_char: usize,
    needle: &str,
    case_sensitive: bool,
) -> Option<Range<usize>> {
    if needle.is_empty() {
        return None;
    }
    let hay: Vec<char> = if case_sensitive {
        text.chars().collect()
    } else {
        text.chars().flat_map(|c| c.to_lowercase()).collect()
    };
    let nd: Vec<char> = if case_sensitive {
        needle.chars().collect()
    } else {
        needle.chars().flat_map(|c| c.to_lowercase()).collect()
    };
    if nd.len() > hay.len() {
        return None;
    }
    let start = start_char.min(hay.len());
    if let Some(r) = scan(&hay, &nd, start) {
        return Some(r);
    }
    // Wrap.
    scan(&hay, &nd, 0)
}

fn scan(hay: &[char], nd: &[char], from: usize) -> Option<Range<usize>> {
    let max = hay.len().saturating_sub(nd.len());
    let mut i = from;
    while i <= max {
        if hay[i..i + nd.len()] == *nd {
            return Some(i..i + nd.len());
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, target: &str) -> usize {
        text.find(target)
            .map(|b| text[..b].chars().count())
            .unwrap_or(0)
    }

    // ── word_at ────────────────────────────────────────────────────

    #[test]
    fn word_at_on_word_char_returns_full_word() {
        let s = "let foo = bar();";
        let r = word_at(s, at(s, "foo") + 1).unwrap();
        assert_eq!(&s.chars().skip(r.start).take(r.end - r.start).collect::<String>(), "foo");
    }

    #[test]
    fn word_at_right_after_word_still_returns_it() {
        let s = "foo bar";
        let r = word_at(s, 3).unwrap();
        assert_eq!(&s.chars().skip(r.start).take(r.end - r.start).collect::<String>(), "foo");
    }

    #[test]
    fn word_at_inside_spaces_returns_none() {
        let s = "a    b";
        assert!(word_at(s, 3).is_none());
    }

    #[test]
    fn word_at_supports_unicode_identifier_chars() {
        let s = "café résumé";
        let r = word_at(s, 2).unwrap();
        let word: String = s.chars().skip(r.start).take(r.end - r.start).collect();
        assert_eq!(word, "café");
    }

    #[test]
    fn word_at_treats_underscore_as_word_char() {
        let s = "snake_case word";
        let r = word_at(s, 0).unwrap();
        let word: String = s.chars().skip(r.start).take(r.end - r.start).collect();
        assert_eq!(word, "snake_case");
    }

    #[test]
    fn word_at_on_empty_text_returns_none() {
        assert!(word_at("", 0).is_none());
    }

    // ── find_next ──────────────────────────────────────────────────

    #[test]
    fn find_next_finds_match_after_cursor() {
        let s = "foo bar foo";
        let r = find_next(s, 4, "foo", true).unwrap();
        // The "foo" at byte 8 starts at char 8.
        assert_eq!(r, 8..11);
    }

    #[test]
    fn find_next_wraps_to_start_when_no_tail_match() {
        let s = "foo bar baz";
        // No "foo" after position 4; should wrap and return the leading one.
        let r = find_next(s, 4, "foo", true).unwrap();
        assert_eq!(r, 0..3);
    }

    #[test]
    fn find_next_case_insensitive_matches_either_case() {
        let s = "Foo bar foo";
        let r = find_next(s, 0, "foo", false).unwrap();
        assert_eq!(r, 0..3);
    }

    #[test]
    fn find_next_case_sensitive_skips_wrong_case() {
        let s = "Foo bar foo";
        let r = find_next(s, 0, "foo", true).unwrap();
        assert_eq!(r, 8..11);
    }

    #[test]
    fn find_next_no_match_returns_none() {
        assert!(find_next("hello", 0, "xyz", true).is_none());
    }

    #[test]
    fn find_next_empty_needle_returns_none() {
        assert!(find_next("hello", 0, "", true).is_none());
    }

    #[test]
    fn find_next_handles_unicode_chars() {
        let s = "café code café";
        let r = find_next(s, 5, "café", true).unwrap();
        // Second café starts at char 10.
        assert_eq!(r, 10..14);
    }
}
