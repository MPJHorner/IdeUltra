//! Crash recovery for dirty buffers.
//!
//! Every dirty `EditorTab` periodically writes its in-memory contents
//! to a recovery directory under the app data path. On a clean save we
//! delete the recovery file. On startup we scan the directory; anything
//! still present represents work that wasn't saved before the previous
//! exit (whether crash, force-quit, or power-loss).
//!
//! Layout under `~/Library/Application Support/com.mpjhorner.IdeUltra/recovery/`:
//!
//! ```
//! <16-hex-hash>.json   metadata: original path, mtime
//! <16-hex-hash>.txt    raw buffer contents (UTF-8)
//! ```
//!
//! The hash is a deterministic DefaultHasher of the canonical path —
//! plenty for collision-avoidance in a per-user recovery dir.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Default debounce between recovery writes for the same buffer.
pub const RECOVERY_DEBOUNCE_MS: u64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMeta {
    pub path: PathBuf,
    /// Seconds since epoch. Coarse but enough to compare against file mtime.
    pub saved_at: u64,
}

#[derive(Debug, Clone)]
pub struct Recovery {
    pub meta: RecoveryMeta,
    pub contents: String,
}

#[derive(Debug, Clone)]
pub struct RecoveryStore {
    dir: PathBuf,
}

impl RecoveryStore {
    pub fn from_project_dirs() -> Option<Self> {
        let dirs = ProjectDirs::from("com", "mpjhorner", "IdeUltra")?;
        let dir = dirs.data_dir().join("recovery");
        Some(Self::new(dir))
    }

    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("mkdir {}", self.dir.display()))
    }

    fn hash_path(path: &Path) -> String {
        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        format!("{:016x}", h.finish())
    }

    fn entry_paths(&self, hash: &str) -> (PathBuf, PathBuf) {
        (
            self.dir.join(format!("{hash}.json")),
            self.dir.join(format!("{hash}.txt")),
        )
    }

    /// Write a recovery snapshot for `original_path` containing `contents`.
    /// Both the metadata and contents files are written atomically (`.tmp`
    /// + rename) so a process death mid-write can't leave a torn pair.
    pub fn write(&self, original_path: &Path, contents: &str) -> Result<()> {
        self.ensure_dir()?;
        let hash = Self::hash_path(original_path);
        let (meta_path, content_path) = self.entry_paths(&hash);

        // Contents first — readers ignore .json without a matching .txt.
        write_atomic(&content_path, contents.as_bytes())?;
        let meta = RecoveryMeta {
            path: original_path.to_path_buf(),
            saved_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        let json = serde_json::to_vec_pretty(&meta)?;
        write_atomic(&meta_path, &json)?;
        Ok(())
    }

    /// Delete the recovery snapshot for `original_path`, if any.
    pub fn clear(&self, original_path: &Path) -> Result<()> {
        let hash = Self::hash_path(original_path);
        let (meta_path, content_path) = self.entry_paths(&hash);
        for p in [&meta_path, &content_path] {
            match fs::remove_file(p) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }
        Ok(())
    }

    /// Return every recoverable buffer in the store.
    ///
    /// A recovery is considered **fresh** and returned when:
    /// - both `.json` and `.txt` exist
    /// - the original file either no longer exists OR its mtime is older
    ///   than the recovery's `saved_at`
    ///
    /// A recovery whose original file is newer is *stale* (the user saved
    /// some other way) and is silently cleaned up.
    pub fn scan(&self) -> Vec<Recovery> {
        let Ok(read) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let hash = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let content_path = self.dir.join(format!("{hash}.txt"));
            if !content_path.is_file() {
                continue;
            }
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let meta: RecoveryMeta = match serde_json::from_slice(&bytes) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if is_recovery_stale(&meta) {
                // Best-effort cleanup; failure here is non-fatal.
                let _ = fs::remove_file(&path);
                let _ = fs::remove_file(&content_path);
                continue;
            }
            let contents = match fs::read_to_string(&content_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            out.push(Recovery { meta, contents });
        }
        out
    }

    /// Delete every recovery in the store. Used when the user dismisses
    /// the startup recovery prompt with "Discard all".
    pub fn clear_all(&self) -> Result<()> {
        let Ok(read) = fs::read_dir(&self.dir) else {
            return Ok(());
        };
        for entry in read.flatten() {
            let _ = fs::remove_file(entry.path());
        }
        Ok(())
    }
}

fn is_recovery_stale(meta: &RecoveryMeta) -> bool {
    let Ok(file_meta) = fs::metadata(&meta.path) else {
        return false; // original gone — keep the recovery, that's the whole point
    };
    let Ok(mtime) = file_meta.modified() else {
        return false;
    };
    let file_secs = mtime
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    file_secs > meta.saved_at
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn store() -> (tempfile::TempDir, RecoveryStore) {
        let dir = tempdir().unwrap();
        let store = RecoveryStore::new(dir.path().join("rec"));
        (dir, store)
    }

    #[test]
    fn write_then_scan_round_trips() {
        let (work, rec) = store();
        let original = work.path().join("file.rs");
        fs::write(&original, "on disk\n").unwrap();

        rec.write(&original, "unsaved edits").unwrap();
        let found = rec.scan();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].contents, "unsaved edits");
        assert_eq!(found[0].meta.path, original);
    }

    #[test]
    fn clear_removes_recovery_pair() {
        let (work, rec) = store();
        let original = work.path().join("file.rs");
        fs::write(&original, "").unwrap();

        rec.write(&original, "x").unwrap();
        assert_eq!(rec.scan().len(), 1);
        rec.clear(&original).unwrap();
        assert!(rec.scan().is_empty());
    }

    #[test]
    fn clear_on_missing_recovery_is_a_noop() {
        let (work, rec) = store();
        let p = work.path().join("nope.rs");
        rec.clear(&p).unwrap(); // must not error
    }

    #[test]
    fn deterministic_hashing_for_same_path() {
        let p = Path::new("/abs/some/file.rs");
        assert_eq!(RecoveryStore::hash_path(p), RecoveryStore::hash_path(p));
        // Different paths produce different hashes (with overwhelming probability).
        let q = Path::new("/abs/other/file.rs");
        assert_ne!(RecoveryStore::hash_path(p), RecoveryStore::hash_path(q));
    }

    #[test]
    fn original_newer_than_recovery_is_filtered_and_cleaned() {
        let (work, rec) = store();
        let original = work.path().join("file.rs");
        fs::write(&original, "old contents").unwrap();

        // Write a recovery that pretends to be from the distant past.
        rec.write(&original, "unsaved").unwrap();
        let hash = RecoveryStore::hash_path(&original);
        let meta_path = &rec.dir.join(format!("{hash}.json"));
        let mut meta: RecoveryMeta =
            serde_json::from_slice(&fs::read(&meta_path).unwrap()).unwrap();
        meta.saved_at = 0; // 1970
        fs::write(&meta_path, serde_json::to_vec(&meta).unwrap()).unwrap();

        // Touch the original so its mtime is "now" (>0).
        fs::write(&original, "newer contents").unwrap();

        let found = rec.scan();
        assert!(found.is_empty(), "stale recovery should be filtered");
        // And the stale pair was cleaned up on scan.
        assert!(!meta_path.exists());
    }

    #[test]
    fn original_missing_keeps_the_recovery() {
        let (work, rec) = store();
        let original = work.path().join("vanished.rs");
        fs::write(&original, "").unwrap();
        rec.write(&original, "unsaved").unwrap();
        fs::remove_file(&original).unwrap();

        let found = rec.scan();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].contents, "unsaved");
    }

    #[test]
    fn scan_ignores_orphan_metadata_without_content() {
        let (_work, rec) = store();
        fs::create_dir_all(&rec.dir).unwrap();
        fs::write(
            &rec.dir.join("ffff.json"),
            br#"{"path":"/x","saved_at":0}"#,
        )
        .unwrap();
        assert!(rec.scan().is_empty());
    }

    #[test]
    fn clear_all_empties_the_store() {
        let (work, rec) = store();
        for n in 0..3 {
            let p = work.path().join(format!("f{n}.rs"));
            fs::write(&p, "").unwrap();
            rec.write(&p, "draft").unwrap();
        }
        assert_eq!(rec.scan().len(), 3);
        rec.clear_all().unwrap();
        assert!(rec.scan().is_empty());
    }
}
