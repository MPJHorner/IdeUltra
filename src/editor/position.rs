//! Pure helpers for cursor position display and line-ending detection.
//! No egui dependency so we can unit-test them cheaply.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Mixed,
    /// Empty buffer or single line with no terminator.
    None,
}

impl LineEnding {
    pub fn label(self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
            LineEnding::Mixed => "Mixed",
            LineEnding::None => "—",
        }
    }
}

/// Detect the line-ending style of `text`. Scans up to 8 KiB to stay
/// fast on huge files — that's plenty for an honest verdict.
pub fn detect_line_ending(text: &str) -> LineEnding {
    let scan_limit = text.len().min(8 * 1024);
    let head = &text.as_bytes()[..scan_limit];
    let mut lf = 0usize;
    let mut crlf = 0usize;
    let mut i = 0;
    while i < head.len() {
        if head[i] == b'\n' {
            if i > 0 && head[i - 1] == b'\r' {
                crlf += 1;
            } else {
                lf += 1;
            }
        }
        i += 1;
    }
    match (lf, crlf) {
        (0, 0) => LineEnding::None,
        (_, 0) => LineEnding::Lf,
        (0, _) => LineEnding::Crlf,
        _ => LineEnding::Mixed,
    }
}

/// 1-based (line, column) for a character index into `text`.
///
/// Column is counted in characters (not bytes, not visual columns).
/// That matches what users expect from a status bar — tabs count as 1.
pub fn line_col_at_char(text: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in text.chars().enumerate() {
        if i == char_index {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Convert a 1-based line number to the character index of its first column.
/// Out-of-range lines clamp to the last line.
pub fn char_index_at_line_start(text: &str, line: usize) -> usize {
    let line = line.max(1);
    let mut current_line = 1usize;
    if current_line == line {
        return 0;
    }
    for (i, ch) in text.chars().enumerate() {
        if ch == '\n' {
            current_line += 1;
            if current_line == line {
                return i + 1;
            }
        }
    }
    // Past EOF: clamp to last line start.
    text.chars().count().saturating_sub(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_has_no_ending() {
        assert_eq!(detect_line_ending(""), LineEnding::None);
    }

    #[test]
    fn single_line_no_terminator_has_no_ending() {
        assert_eq!(detect_line_ending("hello"), LineEnding::None);
    }

    #[test]
    fn lf_detected() {
        assert_eq!(detect_line_ending("a\nb\nc"), LineEnding::Lf);
    }

    #[test]
    fn crlf_detected() {
        assert_eq!(detect_line_ending("a\r\nb\r\nc"), LineEnding::Crlf);
    }

    #[test]
    fn mixed_endings_detected() {
        assert_eq!(detect_line_ending("a\r\nb\nc"), LineEnding::Mixed);
    }

    #[test]
    fn line_col_first_position_is_one_one() {
        assert_eq!(line_col_at_char("abc", 0), (1, 1));
    }

    #[test]
    fn line_col_advances_columns_within_a_line() {
        assert_eq!(line_col_at_char("abc", 2), (1, 3));
    }

    #[test]
    fn line_col_after_newline_resets_column() {
        assert_eq!(line_col_at_char("ab\ncd", 3), (2, 1));
        assert_eq!(line_col_at_char("ab\ncd", 4), (2, 2));
    }

    #[test]
    fn line_col_past_eof_returns_last_known_position() {
        let (l, c) = line_col_at_char("ab", 999);
        // After walking the whole 2-char string, we end at line 1 col 3.
        assert_eq!((l, c), (1, 3));
    }

    #[test]
    fn line_col_with_unicode_counts_chars_not_bytes() {
        // "é" is two bytes but one char.
        assert_eq!(line_col_at_char("é!", 1), (1, 2));
    }

    #[test]
    fn char_index_at_line_start_first_line_is_zero() {
        assert_eq!(char_index_at_line_start("a\nb\nc", 1), 0);
    }

    #[test]
    fn char_index_at_line_start_middle_line() {
        assert_eq!(char_index_at_line_start("a\nb\nc", 2), 2);
        assert_eq!(char_index_at_line_start("a\nb\nc", 3), 4);
    }
}
