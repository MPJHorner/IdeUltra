pub mod tree;
pub mod watcher;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use self::tree::FileTree;
use self::watcher::WorkspaceWatcher;
use crate::finder::FileIndex;
use crate::git::GitStatus;
use std::collections::HashMap;

pub struct Workspace {
    pub root: PathBuf,
    pub tree: FileTree,
    pub watcher: Option<WorkspaceWatcher>,
    /// Lazily populated by the fuzzy finder on first Cmd+P.
    pub file_index: Option<FileIndex>,
    /// Map of absolute path → git status. None when git is unavailable
    /// or the workspace isn't a repo.
    pub git_status: Option<HashMap<PathBuf, GitStatus>>,
}

impl Workspace {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path
            .as_ref()
            .canonicalize()
            .with_context(|| format!("canonicalize {}", path.as_ref().display()))?;
        if !root.is_dir() {
            anyhow::bail!("not a directory: {}", root.display());
        }
        let tree = FileTree::new(&root)?;
        // Watcher failures shouldn't block opening the workspace — log and move on.
        let watcher = match WorkspaceWatcher::watch(&root) {
            Ok(w) => Some(w),
            Err(err) => {
                tracing::warn!(error = %err, "could not start file watcher");
                None
            }
        };
        let mut ws = Self {
            root,
            tree,
            watcher,
            file_index: None,
            git_status: None,
        };
        ws.refresh_git_status();
        Ok(ws)
    }

    pub fn refresh_git_status(&mut self) {
        let before = self
            .git_status
            .as_ref()
            .map(|m| m.len())
            .unwrap_or(0);
        self.git_status = crate::git::read_status(&self.root);
        let after = self
            .git_status
            .as_ref()
            .map(|m| m.len())
            .unwrap_or(0);
        if before != after {
            tracing::debug!(entries = after, "git status refreshed");
        }
    }

    /// Build (or rebuild) the fuzzy-find index for this workspace.
    /// Cheap to call again after watcher events — the walk is fast
    /// and we want the index to stay reasonably fresh.
    pub fn ensure_index(&mut self) {
        if self.file_index.is_none() {
            self.file_index = Some(FileIndex::build(&self.root));
        }
    }

    pub fn invalidate_index(&mut self) {
        self.file_index = None;
    }

    pub fn display_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| self.root.to_str().unwrap_or(""))
            .to_string()
    }
}
