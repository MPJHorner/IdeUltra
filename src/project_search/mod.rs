//! Project-wide text search. Pure functions in here; UI is `ui::project_search_panel`.
//!
//! For MVP we scan synchronously on the UI thread. The caps below keep
//! worst-case scan time bounded; background-thread streaming is a v0.3.

use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::find::{find_matches, FindError, FindOptions};
use crate::finder::FileIndex;

/// Max bytes we'll read per file. Skips large generated files / data dumps.
pub const MAX_FILE_SIZE: u64 = 1_000_000;

/// Hard cap on results to keep the UI snappy.
pub const MAX_RESULTS: usize = 500;

/// One match within a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMatch {
    /// 1-based line number.
    pub line: usize,
    /// Byte range of the match within the *file*, suitable for editor jump.
    pub byte_range: Range<usize>,
    /// The full line text containing the match. Trimmed of trailing newline.
    pub line_text: String,
}

#[derive(Debug, Clone)]
pub struct FileHits {
    pub path: PathBuf,
    pub matches: Vec<LineMatch>,
}

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub hits: Vec<FileHits>,
    pub total_matches: usize,
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub truncated: bool,
    pub error: Option<String>,
}

/// Search every file in `index` for `query`. Returns at most `MAX_RESULTS`
/// matches across all files. Files larger than `MAX_FILE_SIZE` or detected
/// as binary are skipped.
pub fn search_workspace(
    index: &FileIndex,
    query: &str,
    options: FindOptions,
) -> SearchOutcome {
    if query.is_empty() {
        return SearchOutcome {
            hits: Vec::new(),
            total_matches: 0,
            files_scanned: 0,
            files_skipped: 0,
            truncated: false,
            error: None,
        };
    }

    let mut hits: Vec<FileHits> = Vec::new();
    let mut total = 0usize;
    let mut scanned = 0usize;
    let mut skipped = 0usize;
    let mut truncated = false;

    for path in &index.paths {
        if total >= MAX_RESULTS {
            truncated = true;
            break;
        }
        match scan_file(path, query, options) {
            Ok(ScanOutcome::Matches(matches)) => {
                scanned += 1;
                if !matches.is_empty() {
                    total += matches.len();
                    hits.push(FileHits {
                        path: path.clone(),
                        matches,
                    });
                }
            }
            Ok(ScanOutcome::Skipped) => skipped += 1,
            Err(FindError::InvalidRegex(msg)) => {
                return SearchOutcome {
                    hits: Vec::new(),
                    total_matches: 0,
                    files_scanned: scanned,
                    files_skipped: skipped,
                    truncated: false,
                    error: Some(msg),
                };
            }
        }
    }

    SearchOutcome {
        hits,
        total_matches: total,
        files_scanned: scanned,
        files_skipped: skipped,
        truncated,
        error: None,
    }
}

enum ScanOutcome {
    Matches(Vec<LineMatch>),
    Skipped,
}

fn scan_file(path: &Path, query: &str, options: FindOptions) -> Result<ScanOutcome, FindError> {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(ScanOutcome::Skipped),
    };
    if !metadata.is_file() || metadata.len() > MAX_FILE_SIZE {
        return Ok(ScanOutcome::Skipped);
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(ScanOutcome::Skipped),
    };
    if is_binary(&bytes) {
        return Ok(ScanOutcome::Skipped);
    }
    let text = match std::str::from_utf8(&bytes) {
        Ok(t) => t,
        Err(_) => return Ok(ScanOutcome::Skipped),
    };
    let matches = search_text(text, query, options)?;
    Ok(ScanOutcome::Matches(matches))
}

/// Match `query` against the given text and produce one `LineMatch` per
/// match. Public so the unit tests don't need to write to disk.
pub fn search_text(
    text: &str,
    query: &str,
    options: FindOptions,
) -> Result<Vec<LineMatch>, FindError> {
    let ranges = find_matches(text, query, options)?;
    let mut line_starts = vec![0usize];
    for (i, b) in text.as_bytes().iter().enumerate() {
        if *b == b'\n' {
            line_starts.push(i + 1);
        }
    }
    let mut out = Vec::with_capacity(ranges.len());
    for r in ranges {
        let line_idx = match line_starts.binary_search(&r.start) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let line_start = line_starts[line_idx];
        let line_end = line_starts
            .get(line_idx + 1)
            .copied()
            .map(|n| n - 1) // trim the trailing \n
            .unwrap_or(text.len());
        let line_text = text[line_start..line_end].trim_end_matches('\r').to_string();
        out.push(LineMatch {
            line: line_idx + 1,
            byte_range: r,
            line_text,
        });
    }
    Ok(out)
}

fn is_binary(bytes: &[u8]) -> bool {
    // Same heuristic git uses: NUL byte in the first 8 KiB ⇒ binary.
    bytes.iter().take(8 * 1024).any(|b| *b == 0)
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn opts() -> FindOptions {
        FindOptions {
            case_sensitive: true,
            ..Default::default()
        }
    }

    #[test]
    fn search_text_returns_correct_line_numbers() {
        let text = "alpha\nbeta\ngamma\ndelta\n";
        let m = search_text(text, "gamma", opts()).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 3);
        assert_eq!(m[0].line_text, "gamma");
    }

    #[test]
    fn search_text_handles_multiple_matches_on_same_line() {
        let text = "aaa aaa aaa\nzzz\n";
        let m = search_text(text, "aaa", opts()).unwrap();
        assert_eq!(m.len(), 3);
        // All three are on line 1.
        for hit in &m {
            assert_eq!(hit.line, 1);
            assert_eq!(hit.line_text, "aaa aaa aaa");
        }
    }

    #[test]
    fn search_text_trims_carriage_returns() {
        let text = "foo\r\nbar baz\r\n";
        let m = search_text(text, "bar", opts()).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 2);
        assert_eq!(m[0].line_text, "bar baz");
    }

    #[test]
    fn search_text_empty_query_returns_no_matches() {
        let m = search_text("hello", "", opts()).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn search_text_unmatched_returns_empty() {
        let m = search_text("hello world", "xyz", opts()).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn is_binary_detects_null_bytes() {
        assert!(is_binary(b"abc\0def"));
        assert!(!is_binary(b"abc def"));
        // Long pure-text shouldn't flip the heuristic.
        let long: Vec<u8> = (0..10_000).map(|i| b'a' + (i % 26) as u8).collect();
        assert!(!is_binary(&long));
    }

    #[test]
    fn search_workspace_skips_binary_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("text.txt"), "needle in a haystack\n").unwrap();
        fs::write(dir.path().join("binary.dat"), b"prefix\0needle\0suffix").unwrap();

        let idx = FileIndex::build(dir.path());
        let outcome = search_workspace(&idx, "needle", opts());
        assert_eq!(outcome.total_matches, 1);
        assert_eq!(outcome.hits.len(), 1);
        assert!(outcome.hits[0].path.ends_with("text.txt"));
        assert!(outcome.files_skipped >= 1);
    }

    #[test]
    fn search_workspace_aggregates_across_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "foo\nfoo\n").unwrap();
        fs::write(dir.path().join("b.txt"), "foo bar\n").unwrap();
        fs::write(dir.path().join("c.txt"), "nothing here\n").unwrap();

        let idx = FileIndex::build(dir.path());
        let outcome = search_workspace(&idx, "foo", opts());
        assert_eq!(outcome.total_matches, 3);
        assert_eq!(outcome.hits.len(), 2);
        assert!(outcome.error.is_none());
    }

    #[test]
    fn search_workspace_surfaces_invalid_regex_error() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "anything\n").unwrap();
        let idx = FileIndex::build(dir.path());
        let bad_opts = FindOptions {
            case_sensitive: true,
            regex: true,
            ..Default::default()
        };
        let outcome = search_workspace(&idx, "(", bad_opts);
        assert!(outcome.error.is_some());
        assert!(outcome.hits.is_empty());
    }
}
