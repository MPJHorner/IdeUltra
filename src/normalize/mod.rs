//! Pure on-save buffer transformations: strip trailing whitespace on
//! each line, ensure a final newline. Both functions return new owned
//! strings so the caller can decide whether to assign back.

/// Remove trailing whitespace (spaces, tabs, carriage returns) from
/// every line. The final newline (if any) is preserved verbatim — the
/// "trim" only touches whitespace *before* a newline or end-of-file.
pub fn trim_trailing_whitespace(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(text.len());
    for line in text.split('\n') {
        let trimmed = line.trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '\r');
        out.push_str(trimmed);
        out.push('\n');
    }
    // The split produces a phantom trailing empty string when the input
    // ended in '\n', which we just turned into "\n" — so a buffer that
    // already ended with '\n' now ends with "\n\n". Drop the extra.
    if text.ends_with('\n') {
        if out.ends_with("\n\n") {
            out.pop();
        }
    } else {
        // Input did not end with '\n' so we shouldn't have appended one.
        if out.ends_with('\n') {
            out.pop();
        }
    }
    out
}

/// Append a single trailing newline if the text doesn't end with one.
/// No-op on the empty string (an empty file with a single newline is
/// still meaningfully different from genuinely empty).
pub fn ensure_final_newline(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    if text.ends_with('\n') {
        text.to_string()
    } else {
        let mut s = String::with_capacity(text.len() + 1);
        s.push_str(text);
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── trim_trailing_whitespace ────────────────────────────────────

    #[test]
    fn empty_text_stays_empty() {
        assert_eq!(trim_trailing_whitespace(""), "");
    }

    #[test]
    fn strips_trailing_spaces_from_each_line() {
        assert_eq!(
            trim_trailing_whitespace("foo   \nbar\t\nbaz"),
            "foo\nbar\nbaz"
        );
    }

    #[test]
    fn preserves_final_newline_if_present() {
        assert_eq!(trim_trailing_whitespace("foo  \n"), "foo\n");
    }

    #[test]
    fn does_not_add_final_newline_if_absent() {
        assert_eq!(trim_trailing_whitespace("foo  "), "foo");
    }

    #[test]
    fn keeps_blank_lines_blank() {
        // A line that's just spaces becomes empty; surrounding structure
        // is preserved.
        assert_eq!(trim_trailing_whitespace("a\n   \nb"), "a\n\nb");
    }

    #[test]
    fn keeps_leading_whitespace_untouched() {
        assert_eq!(
            trim_trailing_whitespace("    indented   \nx"),
            "    indented\nx"
        );
    }

    #[test]
    fn strips_crlf_carriage_returns() {
        assert_eq!(trim_trailing_whitespace("foo\r\nbar\r\n"), "foo\nbar\n");
    }

    // ── ensure_final_newline ────────────────────────────────────────

    #[test]
    fn empty_text_does_not_grow() {
        assert_eq!(ensure_final_newline(""), "");
    }

    #[test]
    fn appends_newline_when_missing() {
        assert_eq!(ensure_final_newline("hello"), "hello\n");
    }

    #[test]
    fn leaves_existing_newline_alone() {
        assert_eq!(ensure_final_newline("hello\n"), "hello\n");
    }

    #[test]
    fn handles_text_ending_in_blank_line() {
        // Two trailing newlines should stay two.
        assert_eq!(ensure_final_newline("hello\n\n"), "hello\n\n");
    }
}
