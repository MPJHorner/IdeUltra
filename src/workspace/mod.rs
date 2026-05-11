pub mod tree;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use self::tree::FileTree;

pub struct Workspace {
    pub root: PathBuf,
    pub tree: FileTree,
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
        Ok(Self { root, tree })
    }

    pub fn display_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| self.root.to_str().unwrap_or(""))
            .to_string()
    }
}
