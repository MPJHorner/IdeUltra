//! Line-level diff between two text buffers.
//!
//! Used by the external-change banner so users can see what actually
//! changed on disk before deciding to reload. The pure `line_diff()`
//! function is unit-tested; the UI lives in `ui::diff_modal`.

use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    /// Unchanged line — present in both.
    Equal,
    /// Line present only in the new text.
    Added,
    /// Line present only in the old text.
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    /// The line text, sans trailing newline.
    pub text: String,
    /// 1-based line number in the OLD text, if applicable.
    pub old_line: Option<usize>,
    /// 1-based line number in the NEW text, if applicable.
    pub new_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DiffSummary {
    pub lines: Vec<DiffLine>,
    pub added: usize,
    pub removed: usize,
    pub equal: usize,
}

/// Produce a unified line diff between `old` and `new`. Lines are split
/// on `\n`. The summary counts additions, removals, and unchanged lines.
pub fn line_diff(old: &str, new: &str) -> DiffSummary {
    let diff = TextDiff::from_lines(old, new);
    let mut out = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut equal = 0usize;
    let mut old_no = 0usize;
    let mut new_no = 0usize;

    for change in diff.iter_all_changes() {
        let raw = change.value();
        // strip the trailing newline; similar preserves it but we render
        // line-at-a-time so it'd duplicate.
        let text = raw.strip_suffix('\n').unwrap_or(raw).to_string();
        match change.tag() {
            ChangeTag::Equal => {
                old_no += 1;
                new_no += 1;
                equal += 1;
                out.push(DiffLine {
                    kind: DiffKind::Equal,
                    text,
                    old_line: Some(old_no),
                    new_line: Some(new_no),
                });
            }
            ChangeTag::Delete => {
                old_no += 1;
                removed += 1;
                out.push(DiffLine {
                    kind: DiffKind::Removed,
                    text,
                    old_line: Some(old_no),
                    new_line: None,
                });
            }
            ChangeTag::Insert => {
                new_no += 1;
                added += 1;
                out.push(DiffLine {
                    kind: DiffKind::Added,
                    text,
                    old_line: None,
                    new_line: Some(new_no),
                });
            }
        }
    }

    DiffSummary {
        lines: out,
        added,
        removed,
        equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_texts_produce_all_equal() {
        let d = line_diff("a\nb\nc\n", "a\nb\nc\n");
        assert_eq!(d.added, 0);
        assert_eq!(d.removed, 0);
        assert!(d.lines.iter().all(|l| l.kind == DiffKind::Equal));
    }

    #[test]
    fn empty_both_sides_yields_no_lines() {
        let d = line_diff("", "");
        assert!(d.lines.is_empty());
        assert_eq!(d.added, 0);
        assert_eq!(d.removed, 0);
    }

    #[test]
    fn pure_addition_at_end() {
        let d = line_diff("a\nb\n", "a\nb\nc\n");
        assert_eq!(d.added, 1);
        assert_eq!(d.removed, 0);
        let last = d.lines.last().unwrap();
        assert_eq!(last.kind, DiffKind::Added);
        assert_eq!(last.text, "c");
        assert_eq!(last.new_line, Some(3));
        assert!(last.old_line.is_none());
    }

    #[test]
    fn pure_deletion() {
        let d = line_diff("a\nb\nc\n", "a\nc\n");
        assert_eq!(d.added, 0);
        assert_eq!(d.removed, 1);
        let removed_lines: Vec<_> = d
            .lines
            .iter()
            .filter(|l| l.kind == DiffKind::Removed)
            .collect();
        assert_eq!(removed_lines.len(), 1);
        assert_eq!(removed_lines[0].text, "b");
    }

    #[test]
    fn modification_is_remove_plus_add() {
        let d = line_diff("foo\n", "bar\n");
        assert_eq!(d.added, 1);
        assert_eq!(d.removed, 1);
    }

    #[test]
    fn line_numbers_are_1_based_and_in_sequence() {
        let d = line_diff("a\nb\n", "a\nB\nc\n");
        // Expect: Equal(1/1), Remove(2/-), Insert(-/2), Insert(-/3)
        // (similar may merge differently, but old_line and new_line must
        // never decrease within their respective tracks.)
        let mut last_old = 0usize;
        let mut last_new = 0usize;
        for line in &d.lines {
            if let Some(n) = line.old_line {
                assert!(n >= last_old, "old_line went backwards: {n} < {last_old}");
                last_old = n;
            }
            if let Some(n) = line.new_line {
                assert!(n >= last_new, "new_line went backwards: {n} < {last_new}");
                last_new = n;
            }
        }
    }

    #[test]
    fn trailing_newline_is_stripped_from_line_text() {
        let d = line_diff("abc\n", "abc\n");
        assert_eq!(d.lines[0].text, "abc");
    }
}
