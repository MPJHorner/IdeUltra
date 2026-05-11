//! Auto-pair logic for brackets and quotes.
//!
//! Pure functions. The app calls into these from `handle_shortcuts`:
//! when the user types a known opener, we transform a one-character
//! `Event::Text` into a two-character pair and queue a cursor
//! back-step. When they type a closer that already follows the cursor,
//! we suppress the event so the cursor just skips over it.

/// Map opener → (opener, closer). Pairs returned as a 2-char string so
/// callers can write it straight into the buffer.
pub fn pair_for_opener(ch: char) -> Option<(char, char)> {
    match ch {
        '(' => Some(('(', ')')),
        '[' => Some(('[', ']')),
        '{' => Some(('{', '}')),
        '"' => Some(('"', '"')),
        '\'' => Some(('\'', '\'')),
        '`' => Some(('`', '`')),
        _ => None,
    }
}

/// True if `ch` is one of our recognised closers.
pub fn is_closer(ch: char) -> bool {
    matches!(ch, ')' | ']' | '}' | '"' | '\'' | '`')
}

/// True if typing `ch` at character position `cursor` in `text` should
/// be **skipped** (swallowed) because the next character at the cursor
/// is already that closer. e.g. typing `)` right before an existing `)`
/// just moves the cursor past it.
pub fn should_skip_closer(text: &str, cursor_char: usize, ch: char) -> bool {
    if !is_closer(ch) {
        return false;
    }
    text.chars().nth(cursor_char) == Some(ch)
}

/// True if typing an opener at `cursor_char` should auto-pair. We *don't*
/// auto-pair when:
///   * a word character is to the right of the cursor (you're inserting
///     into existing identifier-like text)
///   * for `'` and `"`, when a word character is to the *left* (avoid
///     pairing apostrophes inside `don't`, `it's`)
pub fn should_auto_pair(text: &str, cursor_char: usize, ch: char) -> bool {
    if pair_for_opener(ch).is_none() {
        return false;
    }
    if let Some(right) = text.chars().nth(cursor_char) {
        if is_word_char(right) {
            return false;
        }
    }
    if matches!(ch, '"' | '\'' | '`') && cursor_char > 0 {
        if let Some(left) = text.chars().nth(cursor_char - 1) {
            if is_word_char(left) {
                return false;
            }
        }
    }
    true
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_for_opener_returns_matching_close() {
        assert_eq!(pair_for_opener('('), Some(('(', ')')));
        assert_eq!(pair_for_opener('['), Some(('[', ']')));
        assert_eq!(pair_for_opener('{'), Some(('{', '}')));
        assert_eq!(pair_for_opener('"'), Some(('"', '"')));
        assert_eq!(pair_for_opener('\''), Some(('\'', '\'')));
        assert_eq!(pair_for_opener('`'), Some(('`', '`')));
    }

    #[test]
    fn pair_for_opener_returns_none_for_non_openers() {
        assert!(pair_for_opener('x').is_none());
        assert!(pair_for_opener('+').is_none());
    }

    #[test]
    fn closers_are_recognised() {
        for c in [')', ']', '}', '"', '\'', '`'] {
            assert!(is_closer(c));
        }
        assert!(!is_closer('a'));
        assert!(!is_closer('('));
    }

    #[test]
    fn skip_closer_at_matching_position() {
        // Cursor is at index 4, right before the `)` in "foo()".
        assert!(should_skip_closer("foo()", 4, ')'));
    }

    #[test]
    fn skip_closer_only_when_next_char_matches() {
        // Cursor is at index 0, no char to the right matches.
        assert!(!should_skip_closer("foo", 0, ')'));
    }

    #[test]
    fn skip_closer_ignores_non_closer_inputs() {
        assert!(!should_skip_closer("xyz", 0, 'a'));
    }

    #[test]
    fn auto_pair_inserts_at_end_of_buffer() {
        assert!(should_auto_pair("foo", 3, '('));
    }

    #[test]
    fn auto_pair_skips_when_next_char_is_word() {
        // Typing `(` right before `bar` — likely we're modifying `bar`.
        assert!(!should_auto_pair("bar", 0, '('));
    }

    #[test]
    fn auto_pair_quotes_skip_when_preceded_by_word_char() {
        // "don't" — don't pair the apostrophe.
        assert!(!should_auto_pair("don", 3, '\''));
    }

    #[test]
    fn auto_pair_quotes_pair_at_start_of_word() {
        assert!(should_auto_pair("foo ", 4, '"'));
        assert!(should_auto_pair("foo ", 4, '\''));
    }

    #[test]
    fn auto_pair_bracket_pairs_even_when_preceded_by_word_char() {
        // `fn foo(` — typing `(` after `foo` should still pair to `()`.
        assert!(should_auto_pair("foo", 3, '('));
    }
}
