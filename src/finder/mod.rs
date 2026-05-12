//! Fuzzy file finder for the Cmd+P quick-open modal.
//!
//! Two pieces: an in-memory `FileIndex` of paths under the workspace
//! root, and a pure `score()` function that ranks index entries against
//! a typed query. Both are unit-tested.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// Hard cap on the index size. Keeps memory predictable on monorepos
/// and matching latency consistent. The walker stops once we hit this.
pub const MAX_INDEX_ENTRIES: usize = 50_000;

#[derive(Debug, Clone)]
pub struct FileIndex {
    /// All files in the workspace as full paths — we can open them without
    /// rejoining against the root.
    pub paths: Vec<PathBuf>,
    /// Display strings, parallel to `paths`. Original-case workspace-relative
    /// paths suitable for showing in the UI.
    pub displays: Vec<String>,
    /// Lowercased display strings, parallel to `paths`. Pre-computed so
    /// match scoring doesn't re-lowercase on every keystroke.
    pub haystacks: Vec<String>,
    /// True if the walker stopped early due to MAX_INDEX_ENTRIES.
    pub truncated: bool,
}

impl FileIndex {
    pub fn build(root: &Path) -> Self {
        let mut paths = Vec::new();
        let mut displays = Vec::new();
        let mut haystacks = Vec::new();
        let mut truncated = false;

        let walker = WalkBuilder::new(root)
            .standard_filters(true) // respects .gitignore + .ignore
            .hidden(true) // skip .git, .DS_Store, etc.
            // Honour .gitignore even when the workspace isn't a git repo.
            // Users open arbitrary folders, not just clones.
            .require_git(false)
            .build();

        for entry in walker.flatten() {
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }
            if paths.len() >= MAX_INDEX_ENTRIES {
                truncated = true;
                break;
            }
            let path = entry.into_path();
            let display = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let haystack = display.to_lowercase();
            displays.push(display);
            haystacks.push(haystack);
            paths.push(path);
        }

        Self {
            paths,
            displays,
            haystacks,
            truncated,
        }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

/// Score a candidate string against a query. Higher is better.
/// `None` means no match (the query's characters don't appear in order).
///
/// The scoring rewards:
///   * exact substring matches (large bonus)
///   * matches at the start of the string (prefix bonus)
///   * matches at word boundaries (`/`, `_`, `-`, `.`, camelCase)
///   * consecutive matches
/// And penalises:
///   * gaps between matches
///   * total candidate length (shorter ties win)
pub fn score(candidate_lower: &str, query_lower: &str) -> Option<i32> {
    if query_lower.is_empty() {
        return Some(0);
    }
    if query_lower.len() > candidate_lower.len() {
        return None;
    }
    // Fast-path: contiguous substring is a strong signal.
    if let Some(pos) = candidate_lower.find(query_lower) {
        let mut s = 1000 - (pos as i32 * 2);
        if pos == 0 {
            s += 300; // prefix
        }
        if pos > 0
            && is_boundary(candidate_lower.as_bytes()[pos - 1])
        {
            s += 200; // word-boundary substring
        }
        s -= candidate_lower.len() as i32; // shorter ties win
        return Some(s);
    }

    // Subsequence fuzzy match.
    let cand = candidate_lower.as_bytes();
    let q = query_lower.as_bytes();
    let mut qi = 0usize;
    let mut score = 0i32;
    let mut consecutive = 0i32;
    let mut last_match: Option<usize> = None;

    for (i, &c) in cand.iter().enumerate() {
        if qi >= q.len() {
            break;
        }
        if c == q[qi] {
            let mut contribution = 10;
            if last_match.map_or(false, |li| li + 1 == i) {
                consecutive += 1;
                contribution += consecutive * 8;
            } else {
                consecutive = 0;
            }
            if i == 0 {
                contribution += 30; // first-char prefix
            } else if is_boundary(cand[i - 1]) {
                contribution += 25; // after a separator
            }
            score += contribution;
            last_match = Some(i);
            qi += 1;
        }
    }

    if qi < q.len() {
        return None;
    }

    score -= candidate_lower.len() as i32 / 4;
    Some(score)
}

fn is_boundary(b: u8) -> bool {
    matches!(b, b'/' | b'_' | b'-' | b'.' | b' ')
}

/// Run `score` over the index and return the top `limit` matches.
/// Caller passes `query` as-is; we lowercase once here. Equivalent to
/// `search_with_recent(.., &[], ..)` and kept for tests / external use.
#[allow(dead_code)]
pub fn search(index: &FileIndex, query: &str, limit: usize) -> Vec<Match> {
    search_with_recent(index, query, &[], limit)
}

/// Like `search` but biases results toward recently-opened files.
///
/// Empty query: surfaces recent files first (still capped at `limit`),
/// then fills the remainder with arbitrary index entries.
///
/// Non-empty query: every match gets the regular fuzzy score; matches
/// that are also in `recent` get a bonus inversely proportional to
/// their MRU position. So a recent file with the same raw score as a
/// non-recent file ranks higher.
pub fn search_with_recent(
    index: &FileIndex,
    query: &str,
    recent: &[std::path::PathBuf],
    limit: usize,
) -> Vec<Match> {
    if query.is_empty() {
        return empty_query_results(index, recent, limit);
    }
    let q = query.to_lowercase();
    let recent_bonus: std::collections::HashMap<&std::path::Path, i32> = recent
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_path(), recent_score_bonus(i)))
        .collect();
    let mut hits: Vec<Match> = index
        .haystacks
        .iter()
        .zip(index.displays.iter())
        .zip(index.paths.iter())
        .filter_map(|((h, d), p)| {
            score(h, &q).map(|s| {
                let bonus = recent_bonus.get(p.as_path()).copied().unwrap_or(0);
                (s + bonus, d, p)
            })
        })
        .map(|(s, d, p)| Match {
            score: s,
            path: p.clone(),
            display: d.clone(),
        })
        .collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

fn empty_query_results(
    index: &FileIndex,
    recent: &[std::path::PathBuf],
    limit: usize,
) -> Vec<Match> {
    // Build a lookup so we can yield recent files first (in MRU order)
    // and then top up with anything else.
    let display_by_path: std::collections::HashMap<&std::path::Path, &String> = index
        .paths
        .iter()
        .zip(index.displays.iter())
        .map(|(p, d)| (p.as_path(), d))
        .collect();
    let mut out = Vec::with_capacity(limit);
    let mut seen: std::collections::HashSet<&std::path::Path> = Default::default();
    for r in recent {
        if let Some(display) = display_by_path.get(r.as_path()) {
            out.push(Match {
                score: 0,
                path: r.clone(),
                display: (*display).clone(),
            });
            seen.insert(r.as_path());
            if out.len() >= limit {
                return out;
            }
        }
    }
    // Fill remainder from the natural index order, skipping recents we
    // already surfaced.
    for (p, d) in index.paths.iter().zip(index.displays.iter()) {
        if seen.contains(p.as_path()) {
            continue;
        }
        out.push(Match {
            score: 0,
            path: p.clone(),
            display: d.clone(),
        });
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// MRU-position → score bump. Position 0 (most recent) gets a sizable
/// boost; the bonus tapers off quickly. After ~10 entries the bonus
/// is small enough not to override genuine fuzzy-match quality.
fn recent_score_bonus(mru_pos: usize) -> i32 {
    if mru_pos >= 10 {
        return 5;
    }
    (50 - (mru_pos as i32 * 4)).max(5)
}

#[derive(Debug, Clone)]
pub struct Match {
    pub score: i32,
    pub path: PathBuf,
    /// Original-case workspace-relative path, for display only.
    pub display: String,
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn s(c: &str, q: &str) -> Option<i32> {
        score(&c.to_lowercase(), &q.to_lowercase())
    }

    #[test]
    fn empty_query_scores_zero_for_anything() {
        assert_eq!(s("foo", ""), Some(0));
        assert_eq!(s("", ""), Some(0));
    }

    #[test]
    fn query_longer_than_candidate_doesnt_match() {
        assert_eq!(s("ab", "abcd"), None);
    }

    #[test]
    fn non_subsequence_returns_none() {
        assert_eq!(s("hello", "xyz"), None);
        assert_eq!(s("abc", "ba"), None);
    }

    #[test]
    fn exact_substring_beats_fuzzy_subsequence() {
        let exact = s("src/main.rs", "main").unwrap();
        let fuzzy = s("src/menu/admin/spec.rs", "main").unwrap();
        assert!(
            exact > fuzzy,
            "expected exact ({}) > fuzzy ({})",
            exact,
            fuzzy
        );
    }

    #[test]
    fn prefix_match_outscores_internal_match() {
        let prefix = s("main.rs", "main").unwrap();
        let internal = s("zzz/main.rs", "main").unwrap();
        assert!(prefix > internal);
    }

    #[test]
    fn word_boundary_match_outscores_arbitrary_position() {
        // "/main" should outscore mid-token "amain" because of the boundary.
        let boundary = s("src/main.rs", "main").unwrap();
        let mid = s("src/amain.rs", "main").unwrap();
        assert!(boundary > mid);
    }

    #[test]
    fn case_is_ignored_at_the_caller_layer() {
        // The pure score() takes already-lowercased strings; caller does
        // the lowercasing. This test documents that contract.
        assert!(score("src/main.rs", "MAIN").is_none());
        assert!(score("src/main.rs", "main").is_some());
    }

    #[test]
    fn shorter_candidates_break_ties() {
        let short = s("a.rs", "a").unwrap();
        let long = s("a/very/long/path/to/a.rs", "a").unwrap();
        assert!(short >= long);
    }

    #[test]
    fn build_index_walks_files_only() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("c.rs"), "").unwrap();

        let idx = FileIndex::build(dir.path());
        assert_eq!(idx.len(), 3);
        assert!(!idx.truncated);
    }

    #[test]
    fn build_index_skips_gitignored_paths() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "target/\n*.log\n").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("junk.rs"), "").unwrap();
        fs::write(dir.path().join("debug.log"), "").unwrap();
        fs::write(dir.path().join("src.rs"), "").unwrap();

        let idx = FileIndex::build(dir.path());
        // src.rs + .gitignore (the ignore file itself isn't ignored)
        let names: Vec<String> = idx.haystacks.clone();
        assert!(names.iter().any(|h| h == "src.rs"));
        assert!(!names.iter().any(|h| h.contains("target")));
        assert!(!names.iter().any(|h| h.ends_with(".log")));
    }

    #[test]
    fn search_orders_by_score_and_respects_limit() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src").join("main.rs"), "").unwrap();
        fs::create_dir_all(dir.path().join("deep").join("main")).unwrap();
        fs::write(dir.path().join("deep").join("main").join("z.rs"), "").unwrap();

        let idx = FileIndex::build(dir.path());
        let hits = search(&idx, "main", 2);
        assert_eq!(hits.len(), 2);
        // The two main.rs files (root + src) should rank ahead of deep/main/z.rs
        for h in &hits {
            assert!(h.display.ends_with("main.rs"), "got {}", h.display);
        }
    }

    #[test]
    fn search_empty_query_returns_first_n() {
        let dir = tempdir().unwrap();
        for i in 0..5 {
            fs::write(dir.path().join(format!("f{i}.rs")), "").unwrap();
        }
        let idx = FileIndex::build(dir.path());
        let hits = search(&idx, "", 3);
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn search_with_recent_empty_query_lists_recent_first() {
        let dir = tempdir().unwrap();
        for n in ["a.rs", "b.rs", "c.rs", "d.rs"] {
            fs::write(dir.path().join(n), "").unwrap();
        }
        let idx = FileIndex::build(dir.path());
        let recent = vec![dir.path().join("c.rs"), dir.path().join("a.rs")];
        let hits = search_with_recent(&idx, "", &recent, 10);
        assert_eq!(hits[0].path, recent[0]);
        assert_eq!(hits[1].path, recent[1]);
        assert!(hits.len() >= 4);
    }

    #[test]
    fn search_with_recent_skips_recents_that_arent_in_index() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("real.rs"), "").unwrap();
        let idx = FileIndex::build(dir.path());
        let recent = vec![
            std::path::PathBuf::from("/somewhere/else/gone.rs"),
            dir.path().join("real.rs"),
        ];
        let hits = search_with_recent(&idx, "", &recent, 10);
        // The non-index recent is filtered; the in-index one is first.
        assert_eq!(hits.len(), 1);
        assert!(hits[0].path.ends_with("real.rs"));
    }

    #[test]
    fn search_with_recent_bumps_score_on_ties() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("alpha.rs"), "").unwrap();
        fs::write(dir.path().join("beta.rs"), "").unwrap();
        let idx = FileIndex::build(dir.path());
        // Without bonus, both score identically for ".rs"; the order
        // depends on file walker.
        let recent = vec![dir.path().join("beta.rs")];
        let hits = search_with_recent(&idx, ".rs", &recent, 10);
        // The recent one should rank first regardless of walker order.
        assert!(hits[0].path.ends_with("beta.rs"));
    }

    #[test]
    fn recent_score_bonus_tapers_off() {
        let first = recent_score_bonus(0);
        let middle = recent_score_bonus(5);
        let far = recent_score_bonus(50);
        assert!(first > middle);
        assert!(middle >= far);
        assert!(far >= 5);
    }
}
