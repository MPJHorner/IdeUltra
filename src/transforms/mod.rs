//! Pure text transformations applied to a selection (or the whole buffer
//! if no selection is active). UI dispatch lives in `app::dispatch_command`.

/// Sort the lines of `text` ascending (case-insensitive).
/// Preserves a trailing newline if `text` had one.
pub fn sort_lines(text: &str) -> String {
    sort_lines_inner(text, false)
}

/// Sort the lines of `text` descending (case-insensitive).
pub fn sort_lines_reverse(text: &str) -> String {
    sort_lines_inner(text, true)
}

fn sort_lines_inner(text: &str, reverse: bool) -> String {
    let had_trailing_newline = text.ends_with('\n');
    let mut lines: Vec<&str> = text.split('\n').collect();
    // If there's a trailing newline, the split produces a phantom empty
    // final element — pop it so it doesn't end up at the top after sort.
    let phantom = had_trailing_newline && lines.last() == Some(&"");
    if phantom {
        lines.pop();
    }
    lines.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    if reverse {
        lines.reverse();
    }
    let mut out = lines.join("\n");
    if had_trailing_newline {
        out.push('\n');
    }
    out
}

/// Keep only the first occurrence of each line. Order is preserved
/// (so callers can sort first if they want sorted-unique).
pub fn unique_lines(text: &str) -> String {
    let had_trailing_newline = text.ends_with('\n');
    let mut seen = std::collections::HashSet::new();
    let mut out_lines = Vec::new();
    let mut all: Vec<&str> = text.split('\n').collect();
    let phantom = had_trailing_newline && all.last() == Some(&"");
    if phantom {
        all.pop();
    }
    for line in all {
        if seen.insert(line.to_string()) {
            out_lines.push(line);
        }
    }
    let mut out = out_lines.join("\n");
    if had_trailing_newline {
        out.push('\n');
    }
    out
}

/// Convert each character to uppercase.
pub fn to_upper(text: &str) -> String {
    text.to_uppercase()
}

/// Convert each character to lowercase.
pub fn to_lower(text: &str) -> String {
    text.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_lines_orders_ascending() {
        assert_eq!(sort_lines("c\na\nb"), "a\nb\nc");
    }

    #[test]
    fn sort_lines_is_case_insensitive() {
        let out = sort_lines("Banana\napple\nCHERRY");
        // case-insensitive ordering: apple, Banana, CHERRY
        assert_eq!(out, "apple\nBanana\nCHERRY");
    }

    #[test]
    fn sort_lines_preserves_trailing_newline() {
        assert_eq!(sort_lines("b\na\n"), "a\nb\n");
        assert!(!sort_lines("b\na").ends_with('\n'));
    }

    #[test]
    fn sort_lines_reverse_descends() {
        assert_eq!(sort_lines_reverse("a\nb\nc"), "c\nb\na");
    }

    #[test]
    fn unique_lines_dedupes_preserving_order() {
        let out = unique_lines("a\nb\na\nc\nb");
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn unique_lines_preserves_trailing_newline() {
        let out = unique_lines("a\nb\na\n");
        assert_eq!(out, "a\nb\n");
    }

    #[test]
    fn unique_lines_on_empty_returns_empty() {
        assert_eq!(unique_lines(""), "");
    }

    #[test]
    fn to_upper_and_to_lower_handle_unicode() {
        assert_eq!(to_upper("café"), "CAFÉ");
        assert_eq!(to_lower("ÄÖÜ"), "äöü");
    }

    #[test]
    fn sort_then_unique_yields_sorted_unique() {
        let sorted = sort_lines("c\na\nb\na\nc");
        let result = unique_lines(&sorted);
        assert_eq!(result, "a\nb\nc");
    }
}
