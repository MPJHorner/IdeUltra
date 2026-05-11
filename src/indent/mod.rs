//! Pure indent / dedent over a multi-line selection.
//!
//! Tab / Shift+Tab in the app pre-process to call these functions when
//! the selection spans more than one line. A single-caret Tab still
//! goes through egui's TextEdit (which inserts a literal `\t`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentResult {
    pub new_text: String,
    pub new_range: (usize, usize),
}

/// Indent: prepend `indent` to each line in range. Lines that are
/// entirely blank stay blank — adding leading whitespace to empty
/// lines is noisy and tends to surprise people.
pub fn indent(text: &str, selection: (usize, usize), indent: &str) -> IndentResult {
    transform(text, selection, |line| {
        if line.is_empty() {
            line.to_string()
        } else {
            format!("{indent}{line}")
        }
    })
}

/// Dedent: remove exactly one `indent`-equivalent of leading whitespace
/// per line. Either a single tab or up to `indent.len()` leading spaces.
/// Lines that don't start with whitespace are left untouched.
pub fn dedent(text: &str, selection: (usize, usize), indent: &str) -> IndentResult {
    let unit = indent;
    transform(text, selection, |line| {
        if line.starts_with('\t') {
            line[1..].to_string()
        } else if !unit.is_empty() && line.starts_with(unit) {
            line[unit.len()..].to_string()
        } else {
            // Strip up to `unit.len()` leading spaces (handles tabs/spaces mix).
            let take = line
                .chars()
                .take_while(|c| *c == ' ')
                .take(unit.len())
                .count();
            line[take..].to_string()
        }
    })
}

fn transform<F: Fn(&str) -> String>(
    text: &str,
    selection: (usize, usize),
    f: F,
) -> IndentResult {
    let (sel_start, sel_end) = if selection.0 <= selection.1 {
        selection
    } else {
        (selection.1, selection.0)
    };

    // Convert char-indexed selection bounds to byte indices.
    let start_byte = char_index_to_byte(text, sel_start);
    let end_byte = char_index_to_byte(text, sel_end);
    let line_start_byte = byte_of_line_start(text, start_byte);
    let line_end_byte = byte_of_line_end_inclusive(text, end_byte);

    let head = &text[..line_start_byte];
    let body = &text[line_start_byte..line_end_byte];
    let tail = &text[line_end_byte..];

    let mut lines: Vec<&str> = Vec::new();
    let mut s = 0usize;
    for (i, ch) in body.char_indices() {
        if ch == '\n' {
            lines.push(&body[s..i]);
            s = i + 1;
        }
    }
    lines.push(&body[s..]);

    let mut new_body = String::with_capacity(body.len() + lines.len() * 4);
    let mut first_line_delta: isize = 0;
    let mut total_delta: isize = 0;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            new_body.push('\n');
        }
        let new_line = f(line);
        let delta = new_line.chars().count() as isize - line.chars().count() as isize;
        if i == 0 {
            first_line_delta = delta;
        }
        total_delta += delta;
        new_body.push_str(&new_line);
    }

    let new_text = format!("{head}{new_body}{tail}");
    // For an indent the start should advance by `indent.len()` if the first
    // line was non-empty. For dedent it should retract by however much the
    // first line shrank. `first_line_delta` captures both.
    let new_start = (sel_start as isize + first_line_delta).max(0) as usize;
    let new_end = (sel_end as isize + total_delta).max(new_start as isize) as usize;
    IndentResult {
        new_text,
        new_range: (new_start, new_end),
    }
}

fn char_index_to_byte(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    let mut count = 0usize;
    for (i, _) in text.char_indices() {
        if count == char_index {
            return i;
        }
        count += 1;
    }
    text.len()
}

fn byte_of_line_start(text: &str, byte: usize) -> usize {
    if byte == 0 {
        return 0;
    }
    let head = &text.as_bytes()[..byte];
    head.iter()
        .rposition(|b| *b == b'\n')
        .map(|i| i + 1)
        .unwrap_or(0)
}

fn byte_of_line_end_inclusive(text: &str, byte: usize) -> usize {
    let tail = &text.as_bytes()[byte..];
    match tail.iter().position(|b| *b == b'\n') {
        Some(i) => byte + i, // exclude the newline so we can re-stitch
        None => text.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(text: &str, target: &str) -> usize {
        text.find(target)
            .map(|b| text[..b].chars().count())
            .unwrap_or(0)
    }

    #[test]
    fn indent_single_line_prepends_tab() {
        let r = indent("hello", (0, 5), "\t");
        assert_eq!(r.new_text, "\thello");
    }

    #[test]
    fn indent_multi_line_each_line() {
        let src = "a\nb\nc";
        let r = indent(src, (0, src.chars().count()), "\t");
        assert_eq!(r.new_text, "\ta\n\tb\n\tc");
    }

    #[test]
    fn indent_skips_empty_lines() {
        let src = "one\n\nthree";
        let r = indent(src, (0, src.chars().count()), "    ");
        assert_eq!(r.new_text, "    one\n\n    three");
    }

    #[test]
    fn dedent_removes_leading_tab() {
        let r = dedent("\thello", (0, 6), "    ");
        assert_eq!(r.new_text, "hello");
    }

    #[test]
    fn dedent_removes_full_indent_unit_of_spaces() {
        let r = dedent("    hello", (0, 9), "    ");
        assert_eq!(r.new_text, "hello");
    }

    #[test]
    fn dedent_removes_partial_spaces_when_present() {
        // Only 2 leading spaces present; should strip both.
        let r = dedent("  hello", (0, 7), "    ");
        assert_eq!(r.new_text, "hello");
    }

    #[test]
    fn dedent_no_op_on_unindented_line() {
        let r = dedent("hello", (0, 5), "    ");
        assert_eq!(r.new_text, "hello");
    }

    #[test]
    fn indent_then_dedent_round_trips() {
        let src = "a\nb\nc";
        let len = src.chars().count();
        let i = indent(src, (0, len), "\t");
        let d = dedent(&i.new_text, (0, i.new_text.chars().count()), "\t");
        assert_eq!(d.new_text, src);
    }

    #[test]
    fn selection_starting_mid_line_still_indents_full_line() {
        // Caret is inside "bar" on line 2; selection extends to "baz" on line 3.
        let src = "foo\nbar\nbaz";
        let start = idx(src, "ar");
        let end = idx(src, "baz") + 3;
        let r = indent(src, (start, end), "\t");
        assert!(r.new_text.contains("\tbar"));
        assert!(r.new_text.contains("\tbaz"));
        // Line 1 ("foo") is untouched.
        assert!(r.new_text.starts_with("foo\n"));
    }

    #[test]
    fn unicode_lines_stay_byte_safe() {
        let src = "café\nrésumé";
        let r = indent(src, (0, src.chars().count()), "\t");
        assert_eq!(r.new_text, "\tcafé\n\trésumé");
    }
}
