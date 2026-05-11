//! Most-recently-used list of opened files.
//!
//! Persisted as part of `session.json`. Pushing a path either inserts it
//! at the head (new) or moves an existing entry to the head (revisit).
//! The list is capped so we don't grow forever.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const MAX_RECENT: usize = 30;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentFiles {
    /// Ordered most-recent-first.
    #[serde(default)]
    pub entries: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn push(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.entries.retain(|p| p != &path);
        self.entries.insert(0, path);
        if self.entries.len() > MAX_RECENT {
            self.entries.truncate(MAX_RECENT);
        }
    }

    /// Drop entries whose file no longer exists. Cheap O(n) — call on
    /// startup so the menu doesn't list ghosts.
    pub fn prune_missing(&mut self) {
        self.entries.retain(|p| p.is_file());
    }

    /// Iterate the top `n` most-recent entries.
    pub fn top(&self, n: usize) -> impl Iterator<Item = &Path> {
        self.entries.iter().take(n).map(|p| p.as_path())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn push_inserts_at_head() {
        let mut r = RecentFiles::default();
        r.push(p("/a.rs"));
        r.push(p("/b.rs"));
        assert_eq!(r.entries, vec![p("/b.rs"), p("/a.rs")]);
    }

    #[test]
    fn push_moves_existing_to_head_without_duplicating() {
        let mut r = RecentFiles::default();
        r.push(p("/a.rs"));
        r.push(p("/b.rs"));
        r.push(p("/a.rs")); // revisit a.rs
        assert_eq!(r.entries, vec![p("/a.rs"), p("/b.rs")]);
    }

    #[test]
    fn push_caps_at_max() {
        let mut r = RecentFiles::default();
        for i in 0..(MAX_RECENT + 10) {
            r.push(p(&format!("/f{i}.rs")));
        }
        assert_eq!(r.entries.len(), MAX_RECENT);
        // Most recent push should be at the head.
        let last_pushed = p(&format!("/f{}.rs", MAX_RECENT + 10 - 1));
        assert_eq!(r.entries[0], last_pushed);
    }

    #[test]
    fn top_returns_at_most_n() {
        let mut r = RecentFiles::default();
        for i in 0..5 {
            r.push(p(&format!("/f{i}.rs")));
        }
        let xs: Vec<_> = r.top(3).collect();
        assert_eq!(xs.len(), 3);
    }

    #[test]
    fn prune_missing_drops_nonexistent_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real.rs");
        std::fs::write(&real, "").unwrap();
        let fake = tmp.path().join("ghost.rs");

        let mut r = RecentFiles::default();
        r.push(&real);
        r.push(&fake);
        r.prune_missing();
        assert_eq!(r.entries, vec![real]);
    }
}
