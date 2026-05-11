//! Lightweight git integration: read-only `git status --porcelain` so the
//! sidebar can decorate modified / added / untracked files. No git library
//! dependency — we shell out to the user's git binary.
//!
//! The parser is pure and unit-tested; the I/O wrapper logs and gracefully
//! degrades to "no git data" when anything goes wrong (git missing, not a
//! repo, permission denied, etc.).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflicted,
}

impl GitStatus {
    pub fn marker(self) -> &'static str {
        match self {
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::Untracked => "?",
            GitStatus::Ignored => "!",
            GitStatus::Conflicted => "U",
        }
    }
}

/// Run `git status --porcelain=v1 -z` in `root` and return a map from
/// absolute path → status. Returns `None` if anything goes wrong (git
/// missing, not a repo, command failure). Caller logs.
pub fn read_status(root: &Path) -> Option<HashMap<PathBuf, GitStatus>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain=v1")
        .arg("-z")
        .arg("--untracked-files=normal")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let map = parse_porcelain_z(&output.stdout)
        .into_iter()
        .map(|(rel, s)| (root.join(rel), s))
        .collect();
    Some(map)
}

/// Parse `git status --porcelain=v1 -z` output into (relative path, status).
///
/// `-z` uses NUL separators and never quotes paths, which keeps the parser
/// simple. Rename entries are two NUL-terminated paths in a row:
/// `R<sp><new>\0<old>\0`. We only keep the *new* path.
pub fn parse_porcelain_z(bytes: &[u8]) -> Vec<(PathBuf, GitStatus)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Each record begins with two status bytes + space + path, then NUL.
        if i + 3 > bytes.len() {
            break;
        }
        let xy = &bytes[i..i + 2];
        let is_rename = xy[0] == b'R' || xy[1] == b'R';
        // Skip "XY ".
        let path_start = i + 3;
        let path_end = match bytes[path_start..].iter().position(|b| *b == 0) {
            Some(n) => path_start + n,
            None => break,
        };
        let path = bytes_to_path(&bytes[path_start..path_end]);
        let status = classify(xy);
        out.push((path, status));
        i = path_end + 1;
        if is_rename {
            // Consume the old path that follows.
            let old_end = match bytes[i..].iter().position(|b| *b == 0) {
                Some(n) => i + n,
                None => break,
            };
            i = old_end + 1;
        }
    }
    out
}

fn classify(xy: &[u8]) -> GitStatus {
    if xy[0] == b'U' || xy[1] == b'U' || (xy[0] == b'A' && xy[1] == b'A') {
        return GitStatus::Conflicted;
    }
    match (xy[0], xy[1]) {
        (b'?', b'?') => GitStatus::Untracked,
        (b'!', b'!') => GitStatus::Ignored,
        (b'A', _) | (_, b'A') => GitStatus::Added,
        (b'D', _) | (_, b'D') => GitStatus::Deleted,
        (b'R', _) | (_, b'R') => GitStatus::Renamed,
        (b'M', _) | (_, b'M') => GitStatus::Modified,
        _ => GitStatus::Modified,
    }
}

fn bytes_to_path(b: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(std::ffi::OsStr::from_bytes(b))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf8_lossy(b).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(s: &str) -> Vec<u8> {
        // Convenience: replace \\0 in the test string with actual NUL bytes.
        s.replace("\\0", "\0").into_bytes()
    }

    #[test]
    fn empty_input_yields_no_entries() {
        assert!(parse_porcelain_z(&[]).is_empty());
    }

    #[test]
    fn untracked_file_classified_as_untracked() {
        let bytes = z("?? new.txt\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(parsed, vec![(PathBuf::from("new.txt"), GitStatus::Untracked)]);
    }

    #[test]
    fn worktree_modified_file_classified_as_modified() {
        let bytes = z(" M src/main.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(
            parsed,
            vec![(PathBuf::from("src/main.rs"), GitStatus::Modified)]
        );
    }

    #[test]
    fn staged_addition_classified_as_added() {
        let bytes = z("A  src/new.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(
            parsed,
            vec![(PathBuf::from("src/new.rs"), GitStatus::Added)]
        );
    }

    #[test]
    fn deletion_classified_as_deleted() {
        let bytes = z(" D removed.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(
            parsed,
            vec![(PathBuf::from("removed.rs"), GitStatus::Deleted)]
        );
    }

    #[test]
    fn rename_record_consumes_both_paths_and_keeps_new() {
        // R<sp>new\0old\0
        let bytes = z("R  src/new.rs\\0src/old.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, PathBuf::from("src/new.rs"));
        assert_eq!(parsed[0].1, GitStatus::Renamed);
    }

    #[test]
    fn unmerged_files_classified_as_conflicted() {
        let bytes = z("UU conflict.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(parsed[0].1, GitStatus::Conflicted);
    }

    #[test]
    fn multiple_records_parsed_in_order() {
        let bytes = z("?? a.rs\\0 M b.rs\\0A  c.rs\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(
            parsed,
            vec![
                (PathBuf::from("a.rs"), GitStatus::Untracked),
                (PathBuf::from("b.rs"), GitStatus::Modified),
                (PathBuf::from("c.rs"), GitStatus::Added),
            ]
        );
    }

    #[test]
    fn paths_with_spaces_pass_through_unchanged() {
        // -z disables quoting; spaces are preserved as-is.
        let bytes = z(" M docs/my notes.md\\0");
        let parsed = parse_porcelain_z(&bytes);
        assert_eq!(parsed[0].0, PathBuf::from("docs/my notes.md"));
    }

    #[test]
    fn marker_glyphs_are_one_letter() {
        for s in [
            GitStatus::Modified,
            GitStatus::Added,
            GitStatus::Deleted,
            GitStatus::Renamed,
            GitStatus::Untracked,
            GitStatus::Ignored,
            GitStatus::Conflicted,
        ] {
            assert_eq!(s.marker().chars().count(), 1);
        }
    }
}
