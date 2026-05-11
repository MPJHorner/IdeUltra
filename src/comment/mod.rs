//! Pure line-comment toggle.
//!
//! Given a buffer, a character range covering the user's selection, and
//! the language's line-comment token, return a new buffer plus the new
//! selection (so the UI can restore the caret after the edit).
//!
//! Toggle semantics match VS Code:
//!   * If **every** non-empty line in range is commented, uncomment.
//!   * Otherwise, comment every non-empty line.
//! Indented lines are commented after their leading whitespace.

use syntect::parsing::SyntaxReference;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleResult {
    pub new_text: String,
    /// New caret range in chars (start, end), 0-based.
    pub new_range: (usize, usize),
}

/// Token used to start a line comment for a given syntect syntax. None if
/// the language doesn't have one we know about — caller should no-op.
pub fn line_comment_token(syntax: &SyntaxReference) -> Option<&'static str> {
    // Match by syntect's `name` so the mapping stays stable across syntaxes.
    match syntax.name.as_str() {
        "Rust" | "JavaScript" | "TypeScript" | "TSX" | "JavaScript (Babel)"
        | "Go" | "C" | "C++" | "C#" | "Java" | "Kotlin" | "Scala" | "Swift"
        | "PHP" | "Dart" | "Objective-C" | "Objective-C++" | "Groovy"
        | "JSON" | "JSON (with comments)" | "JSON5" => Some("//"),
        "Python" | "Ruby" | "Shell-Unix-Generic" | "Bash" | "YAML"
        | "TOML" | "Makefile" | "Dockerfile" | "R" | "Perl" | "Crystal"
        | "Elixir" | "Nix" | "CMake" | "Ini" | "INI" | "Properties" => Some("#"),
        "Lua" | "Haskell" | "SQL" | "Elm" | "ada" | "VHDL" => Some("--"),
        "HTML" | "XML" | "Markdown" | "MultiMarkdown" => Some("<!--"),
        "CSS" | "SCSS" | "Sass" | "Less" => Some("/*"),
        _ => None,
    }
}

/// Apply a toggle over the byte range `[start_char, end_char)` (1-line
/// minimum). The range is given in *character* indices, matching
/// egui's `CCursor`. Returns the new buffer + new selection.
pub fn toggle_line_comment(
    text: &str,
    selection: (usize, usize),
    token: &str,
) -> ToggleResult {
    if !is_simple_line_token(token) {
        // Wrap-style (e.g. `<!-- … -->`) is a separate operation; we
        // fall through to a no-op rather than corrupting the buffer.
        return ToggleResult {
            new_text: text.to_string(),
            new_range: selection,
        };
    }
    let (sel_start, sel_end) = normalize_range(selection);
    let (line_start_char, line_end_char) =
        expand_to_line_bounds(text, sel_start, sel_end);

    // Slice out the affected lines.
    let line_start_byte = char_index_to_byte(text, line_start_char);
    let line_end_byte = char_index_to_byte(text, line_end_char);
    let head = &text[..line_start_byte];
    let body = &text[line_start_byte..line_end_byte];
    let tail = &text[line_end_byte..];

    // Split into individual lines (without the trailing \n on each).
    let mut lines: Vec<&str> = Vec::new();
    let mut start = 0usize;
    for (i, ch) in body.char_indices() {
        if ch == '\n' {
            lines.push(&body[start..i]);
            start = i + 1;
        }
    }
    // The final trailing slice (no newline) — always include it, even if empty.
    lines.push(&body[start..]);

    // Decide direction: if every non-empty line begins with the token
    // (possibly after whitespace), we uncomment. Otherwise we comment.
    let all_commented = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .all(|l| line_is_commented(l, token));

    let mut transformed = String::with_capacity(body.len() + lines.len() * (token.len() + 2));
    let mut delta_chars: isize = 0;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            transformed.push('\n');
        }
        let (new_line, line_delta) = if all_commented {
            uncomment_line(line, token)
        } else if !line.trim().is_empty() {
            comment_line(line, token)
        } else {
            (line.to_string(), 0)
        };
        transformed.push_str(&new_line);
        delta_chars += line_delta;
    }

    let new_text = format!("{head}{transformed}{tail}");
    // Adjust caret/selection by the net characters added/removed.
    let new_end = (sel_end as isize + delta_chars).max(sel_start as isize) as usize;
    ToggleResult {
        new_text,
        new_range: (sel_start, new_end),
    }
}

fn is_simple_line_token(token: &str) -> bool {
    // Only support simple prefix tokens for now. `<!--` / `/*` need block
    // toggling, which we'll add later.
    !token.contains("<!--") && !token.contains("/*")
}

fn line_is_commented(line: &str, token: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(token)
}

/// Returns the line with the token inserted after leading whitespace, and
/// the number of characters added (token.chars().count() + 1 for the space).
fn comment_line(line: &str, token: &str) -> (String, isize) {
    let leading: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let body = &line[leading.len()..];
    let inserted = format!("{token} ");
    let new_line = format!("{leading}{inserted}{body}");
    (new_line, inserted.chars().count() as isize)
}

/// Returns the line with the token (and trailing single space if any)
/// removed, and the number of characters removed as a negative delta.
fn uncomment_line(line: &str, token: &str) -> (String, isize) {
    let leading_len: usize = line
        .chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| c.len_utf8())
        .sum();
    let leading = &line[..leading_len];
    let after = &line[leading_len..];
    if !after.starts_with(token) {
        return (line.to_string(), 0);
    }
    let after_token = &after[token.len()..];
    let (removed_text, after_space) = if after_token.starts_with(' ') {
        (format!("{token} "), &after_token[1..])
    } else {
        (token.to_string(), after_token)
    };
    let new_line = format!("{leading}{after_space}");
    (new_line, -(removed_text.chars().count() as isize))
}

fn normalize_range((a, b): (usize, usize)) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

fn expand_to_line_bounds(text: &str, start: usize, end: usize) -> (usize, usize) {
    let mut line_start = 0usize;
    let mut chars = 0usize;
    let mut iter = text.char_indices().peekable();
    for (_, ch) in iter.by_ref() {
        if chars == start {
            break;
        }
        if ch == '\n' {
            line_start = chars + 1;
        }
        chars += 1;
    }
    // After the loop, `chars` is at the selection start; rewind to the
    // start of THIS line by reusing the saved `line_start`.
    let mut line_end = end;
    let mut c = chars;
    for ch in text.chars().skip(c) {
        if c >= end && ch == '\n' {
            break;
        }
        c += 1;
        line_end = c;
        if ch == '\n' && c >= end {
            // Include trailing newline of the last line in the selection.
            break;
        }
    }
    // Don't include the trailing '\n' in our slice; we'll re-join.
    if line_end > line_start && text.chars().nth(line_end - 1) == Some('\n') {
        line_end -= 1;
    }
    (line_start, line_end)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_token() -> &'static str {
        "//"
    }

    #[test]
    fn plain_line_gets_commented() {
        let r = toggle_line_comment("let x = 1;", (0, 0), rust_token());
        assert_eq!(r.new_text, "// let x = 1;");
    }

    #[test]
    fn commented_line_gets_uncommented() {
        let r = toggle_line_comment("// let x = 1;", (0, 0), rust_token());
        assert_eq!(r.new_text, "let x = 1;");
    }

    #[test]
    fn commented_with_no_space_after_token_still_uncomments() {
        let r = toggle_line_comment("//hidden", (0, 0), rust_token());
        assert_eq!(r.new_text, "hidden");
    }

    #[test]
    fn indented_line_is_commented_after_whitespace() {
        let r = toggle_line_comment("    foo()", (0, 0), rust_token());
        assert_eq!(r.new_text, "    // foo()");
    }

    #[test]
    fn indented_comment_is_uncommented_after_whitespace() {
        let r = toggle_line_comment("    // foo()", (0, 0), rust_token());
        assert_eq!(r.new_text, "    foo()");
    }

    #[test]
    fn multi_line_selection_commented_uniformly() {
        let src = "a\nb\nc";
        let r = toggle_line_comment(src, (0, src.chars().count()), rust_token());
        assert_eq!(r.new_text, "// a\n// b\n// c");
    }

    #[test]
    fn multi_line_all_commented_uncomments_uniformly() {
        let src = "// a\n// b\n// c";
        let r = toggle_line_comment(src, (0, src.chars().count()), rust_token());
        assert_eq!(r.new_text, "a\nb\nc");
    }

    #[test]
    fn mixed_lines_get_all_commented() {
        // VS Code semantics: if any non-empty line is uncommented, comment all.
        let src = "// a\nb\n// c";
        let r = toggle_line_comment(src, (0, src.chars().count()), rust_token());
        assert_eq!(r.new_text, "// // a\n// b\n// // c");
    }

    #[test]
    fn empty_lines_are_skipped() {
        let src = "a\n\nb";
        let r = toggle_line_comment(src, (0, src.chars().count()), rust_token());
        assert_eq!(r.new_text, "// a\n\n// b");
    }

    #[test]
    fn python_uses_hash_token() {
        let r = toggle_line_comment("x = 1", (0, 0), "#");
        assert_eq!(r.new_text, "# x = 1");
        let r2 = toggle_line_comment("# x = 1", (0, 0), "#");
        assert_eq!(r2.new_text, "x = 1");
    }

    #[test]
    fn unsupported_wrap_token_is_a_noop() {
        let r = toggle_line_comment("hello", (0, 5), "<!--");
        assert_eq!(r.new_text, "hello");
    }

    #[test]
    fn unicode_lines_preserve_byte_safety() {
        let src = "café\nrésumé";
        let r = toggle_line_comment(src, (0, src.chars().count()), rust_token());
        assert_eq!(r.new_text, "// café\n// résumé");
    }
}
