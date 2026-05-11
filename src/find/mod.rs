//! Pure find/replace logic. UI lives in `ui::find_bar`.
//!
//! The functions here take `&str` and return byte ranges. No egui types,
//! no I/O — so they're cheap to unit-test, which is exactly what we do.

use std::ops::Range;

use regex::{Regex, RegexBuilder};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FindOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
}

#[derive(Debug)]
pub enum FindError {
    InvalidRegex(String),
}

/// Find every (non-overlapping) match of `query` in `text`.
///
/// Returns byte ranges that are safe to use for slicing `text` and for
/// turning into egui `CCursor`s after a `char_indices()` conversion.
pub fn find_matches(
    text: &str,
    query: &str,
    options: FindOptions,
) -> Result<Vec<Range<usize>>, FindError> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let re = compile(query, options)?;
    Ok(re.find_iter(text).map(|m| m.range()).collect())
}

/// Replace **one** match starting at `start` with `replacement`.
/// Returns the new text and the byte length of the replacement
/// (so the caller can re-position the cursor).
pub fn replace_one(
    text: &str,
    matched: Range<usize>,
    replacement: &str,
) -> (String, usize) {
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..matched.start]);
    out.push_str(replacement);
    out.push_str(&text[matched.end..]);
    (out, replacement.len())
}

/// Replace every non-overlapping match. Returns the new text and the count.
pub fn replace_all(
    text: &str,
    query: &str,
    replacement: &str,
    options: FindOptions,
) -> Result<(String, usize), FindError> {
    if query.is_empty() {
        return Ok((text.to_string(), 0));
    }
    let re = compile(query, options)?;
    let mut count = 0;
    let mut out = String::with_capacity(text.len());
    let mut last_end = 0;
    for m in re.find_iter(text) {
        out.push_str(&text[last_end..m.start()]);
        out.push_str(replacement);
        last_end = m.end();
        count += 1;
    }
    out.push_str(&text[last_end..]);
    Ok((out, count))
}

fn compile(query: &str, options: FindOptions) -> Result<Regex, FindError> {
    let pattern = if options.regex {
        query.to_string()
    } else if options.whole_word {
        format!(r"\b{}\b", regex::escape(query))
    } else {
        regex::escape(query)
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(!options.case_sensitive)
        .build()
        .map_err(|e| FindError::InvalidRegex(e.to_string()))
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges<'a>(text: &'a str, query: &str, opts: FindOptions) -> Vec<&'a str> {
        find_matches(text, query, opts)
            .unwrap()
            .into_iter()
            .map(|r| &text[r])
            .collect()
    }

    #[test]
    fn empty_query_returns_no_matches() {
        let m = find_matches("hello", "", FindOptions::default()).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn plain_substring_case_insensitive_by_default() {
        let opts = FindOptions::default();
        let m = ranges("The quick brown fox\nThe lazy dog", "the", opts);
        assert_eq!(m, vec!["The", "The"]);
    }

    #[test]
    fn case_sensitive_only_matches_exact_case() {
        let opts = FindOptions {
            case_sensitive: true,
            ..Default::default()
        };
        let m = ranges("The quick THE", "The", opts);
        assert_eq!(m, vec!["The"]);
    }

    #[test]
    fn whole_word_matches_word_boundary_only() {
        let opts = FindOptions {
            case_sensitive: true,
            whole_word: true,
            ..Default::default()
        };
        // "cat" is matched, "scattered" is not, "cat." is matched.
        let m = ranges("cat scattered cat.", "cat", opts);
        assert_eq!(m, vec!["cat", "cat"]);
    }

    #[test]
    fn regex_mode_accepts_pattern() {
        let opts = FindOptions {
            case_sensitive: true,
            regex: true,
            ..Default::default()
        };
        let m = ranges("foo123 bar 42 baz9", r"\d+", opts);
        assert_eq!(m, vec!["123", "42", "9"]);
    }

    #[test]
    fn regex_special_chars_are_escaped_when_not_in_regex_mode() {
        // `.` is a literal dot when regex mode is off.
        let opts = FindOptions::default();
        let m = ranges("a.b.c abc", ".", opts);
        assert_eq!(m, vec![".", "."]);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let opts = FindOptions {
            regex: true,
            ..Default::default()
        };
        let err = find_matches("hello", "(", opts);
        assert!(matches!(err, Err(FindError::InvalidRegex(_))));
    }

    #[test]
    fn replace_one_replaces_only_the_given_range() {
        let (out, len) = replace_one("the cat sat", 4..7, "dog");
        assert_eq!(out, "the dog sat");
        assert_eq!(len, 3);
    }

    #[test]
    fn replace_all_counts_replacements() {
        let opts = FindOptions {
            case_sensitive: true,
            ..Default::default()
        };
        let (out, n) = replace_all("a b a b a", "a", "X", opts).unwrap();
        assert_eq!(out, "X b X b X");
        assert_eq!(n, 3);
    }

    #[test]
    fn replace_all_with_empty_query_is_noop() {
        let (out, n) = replace_all("hello", "", "X", FindOptions::default()).unwrap();
        assert_eq!(out, "hello");
        assert_eq!(n, 0);
    }

    #[test]
    fn unicode_matches_are_byte_safe_slices() {
        // Two-byte UTF-8 chars surround the match. The returned range must
        // be a valid string slice — i.e. the test below must not panic.
        let opts = FindOptions {
            case_sensitive: true,
            ..Default::default()
        };
        let text = "café code café";
        let m = find_matches(text, "code", opts).unwrap();
        assert_eq!(m.len(), 1);
        let r = m[0].clone();
        assert_eq!(&text[r], "code");
    }
}
