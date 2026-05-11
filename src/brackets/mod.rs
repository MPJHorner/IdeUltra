//! Pure bracket-matching for the editor's highlight pass.
//!
//! Given a buffer and a caret position (byte index), find the pair of
//! brackets the caret is **adjacent to** (or sitting on). Returns
//! `(open_byte, close_byte)` regardless of whether the caret is at
//! the opener or the closer side.
//!
//! Strings, comments, and escaped brackets are out of scope — handling
//! them needs real lexing, and the visual cost of an occasional
//! false-positive in a string is minor. This is good enough for a
//! preview-style decoration.

const OPENERS: &[char] = &['(', '[', '{'];
const CLOSERS: &[char] = &[')', ']', '}'];

fn match_pair(c: char) -> Option<char> {
    match c {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        ')' => Some('('),
        ']' => Some('['),
        '}' => Some('{'),
        _ => None,
    }
}

/// Find the bracket pair the cursor is touching, if any.
///
/// `cursor_byte` is a byte index into `text` (e.g. from
/// `egui::CCursor` → char_index_to_byte). The caret is considered to
/// "touch" the bracket at the position `cursor_byte` *or* the one
/// immediately before it — matching VS Code behaviour, where placing
/// the caret right after a `)` highlights its `(`.
pub fn find_matching(text: &str, cursor_byte: usize) -> Option<(usize, usize)> {
    let pos = bracket_position_near(text, cursor_byte)?;
    let bytes = text.as_bytes();
    let ch = bytes[pos] as char;
    if OPENERS.contains(&ch) {
        find_forward(text, pos, ch, match_pair(ch)?).map(|c| (pos, c))
    } else if CLOSERS.contains(&ch) {
        find_backward(text, pos, ch, match_pair(ch)?).map(|o| (o, pos))
    } else {
        None
    }
}

/// Returns the byte index of a bracket the cursor is touching.
/// Prefers the byte at `cursor_byte`; falls back to `cursor_byte - 1`.
fn bracket_position_near(text: &str, cursor_byte: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if cursor_byte < bytes.len() {
        let c = bytes[cursor_byte] as char;
        if OPENERS.contains(&c) || CLOSERS.contains(&c) {
            return Some(cursor_byte);
        }
    }
    if cursor_byte > 0 {
        let prev = cursor_byte - 1;
        if prev < bytes.len() {
            let c = bytes[prev] as char;
            if OPENERS.contains(&c) || CLOSERS.contains(&c) {
                return Some(prev);
            }
        }
    }
    None
}

fn find_forward(text: &str, start: usize, opener: char, closer: char) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1i32;
    let mut i = start + 1;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == opener {
            depth += 1;
        } else if c == closer {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn find_backward(text: &str, start: usize, closer: char, opener: char) -> Option<usize> {
    if start == 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut depth = 1i32;
    let mut i = start;
    while i > 0 {
        i -= 1;
        let c = bytes[i] as char;
        if c == closer {
            depth += 1;
        } else if c == opener {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, target: char, nth: usize) -> usize {
        text.char_indices()
            .filter(|(_, c)| *c == target)
            .nth(nth)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    #[test]
    fn no_match_when_cursor_not_on_or_next_to_bracket() {
        assert!(find_matching("abc def", 2).is_none());
    }

    #[test]
    fn paren_pair_found_from_opener() {
        let src = "fn foo()";
        let open = at(src, '(', 0);
        let close = at(src, ')', 0);
        assert_eq!(find_matching(src, open), Some((open, close)));
    }

    #[test]
    fn paren_pair_found_from_closer() {
        let src = "fn foo()";
        let close = at(src, ')', 0);
        assert_eq!(find_matching(src, close), Some((at(src, '(', 0), close)));
    }

    #[test]
    fn cursor_right_after_closer_still_matches() {
        // VS Code convention: caret at position closer+1 still highlights.
        let src = "()x";
        let close = at(src, ')', 0);
        let cursor = close + 1; // right after `)`
        assert_eq!(find_matching(src, cursor), Some((at(src, '(', 0), close)));
    }

    #[test]
    fn nested_brackets_match_correctly() {
        let src = "((a))";
        // Outer `(`
        let outer_open = 0;
        let outer_close = src.len() - 1;
        assert_eq!(find_matching(src, outer_open), Some((outer_open, outer_close)));
        // Inner `(`
        let inner_open = 1;
        let inner_close = src.len() - 2;
        assert_eq!(find_matching(src, inner_open), Some((inner_open, inner_close)));
    }

    #[test]
    fn square_and_curly_brackets_match() {
        let src = "[a]{b}";
        let lb = at(src, '[', 0);
        let rb = at(src, ']', 0);
        assert_eq!(find_matching(src, lb), Some((lb, rb)));
        let lc = at(src, '{', 0);
        let rc = at(src, '}', 0);
        assert_eq!(find_matching(src, lc), Some((lc, rc)));
    }

    #[test]
    fn different_bracket_kinds_do_not_match_each_other() {
        // `(]` doesn't form a pair.
        let src = "(]";
        assert_eq!(find_matching(src, 0), None);
    }

    #[test]
    fn unbalanced_opener_returns_none() {
        let src = "(abc";
        assert_eq!(find_matching(src, 0), None);
    }

    #[test]
    fn unbalanced_closer_returns_none() {
        let src = "abc)";
        assert_eq!(find_matching(src, 3), None);
    }

    #[test]
    fn deeply_nested_balances() {
        let src = "[[[[]]]]";
        // Outermost `[` is at index 0; matching `]` is at 7.
        assert_eq!(find_matching(src, 0), Some((0, 7)));
        // Innermost pair is at indices 3 & 4.
        assert_eq!(find_matching(src, 3), Some((3, 4)));
    }

    #[test]
    fn unicode_chars_between_brackets_do_not_confuse_byte_scan() {
        // Brackets themselves are ASCII; intermediate Unicode is fine
        // because we only compare bytes for `(`, `)`, etc.
        let src = "(café résumé)";
        let close = src.find(')').unwrap();
        assert_eq!(find_matching(src, 0), Some((0, close)));
    }
}
