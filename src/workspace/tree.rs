use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// A lazy file tree. Directory children are loaded on first expand
/// so that workspaces with tens of thousands of files don't pay
/// the walk cost on open.
pub struct FileTree {
    pub root: TreeNode,
    gitignore: Gitignore,
    workspace_root: PathBuf,
}

pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// `None` = not yet loaded (only meaningful for directories).
    pub children: Option<Vec<TreeNode>>,
    pub expanded: bool,
}

impl FileTree {
    pub fn new(root: &Path) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(root);
        let gi_path = root.join(".gitignore");
        if gi_path.is_file() {
            // Errors here are non-fatal; we just lose ignore rules.
            let _ = builder.add(&gi_path);
        }
        let gitignore = builder
            .build()
            .unwrap_or_else(|_| Gitignore::empty());

        let mut root_node = TreeNode {
            name: root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            path: root.to_path_buf(),
            is_dir: true,
            children: None,
            expanded: true,
        };

        let mut tree = Self {
            root: root_node_placeholder(&mut root_node),
            gitignore,
            workspace_root: root.to_path_buf(),
        };
        // Eagerly load the root one level so the user sees something.
        tree.load_children(&[]);
        Ok(tree)
    }

    /// Load children for the node at `path` (an index path from root).
    /// Idempotent: cheap to call on already-loaded nodes.
    pub fn load_children(&mut self, path: &[usize]) {
        // Read the target dir path & whether it's already loaded without
        // holding the mutable borrow across the filesystem call.
        let (target_path, already_loaded) = {
            let node = match self.node_at_mut(path) {
                Some(n) => n,
                None => return,
            };
            if !node.is_dir {
                return;
            }
            (node.path.clone(), node.children.is_some())
        };
        if already_loaded {
            return;
        }
        let children = read_dir_sorted(&target_path, &self.workspace_root, &self.gitignore);
        if let Some(node) = self.node_at_mut(path) {
            node.children = Some(children);
        }
    }

    pub fn toggle(&mut self, path: &[usize]) {
        // Ensure loaded then flip.
        self.load_children(path);
        if let Some(node) = self.node_at_mut(path) {
            node.expanded = !node.expanded;
        }
    }

    pub fn node_at_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        let mut node = &mut self.root;
        for idx in path {
            let children = node.children.as_mut()?;
            node = children.get_mut(*idx)?;
        }
        Some(node)
    }
}

fn root_node_placeholder(node: &mut TreeNode) -> TreeNode {
    TreeNode {
        name: std::mem::take(&mut node.name),
        path: std::mem::replace(&mut node.path, PathBuf::new()),
        is_dir: node.is_dir,
        children: None,
        expanded: node.expanded,
    }
}

fn read_dir_sorted(
    dir: &Path,
    workspace_root: &Path,
    gitignore: &Gitignore,
) -> Vec<TreeNode> {
    let mut entries: Vec<TreeNode> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();

                // Always hide .git and common heavy dirs at any level,
                // even if not in .gitignore. Cheap and saves users from themselves.
                if matches!(name.as_str(), ".git" | ".DS_Store") {
                    return None;
                }

                let ft = e.file_type().ok()?;
                let is_dir = ft.is_dir();

                // Honour the workspace .gitignore.
                let rel = path.strip_prefix(workspace_root).unwrap_or(&path);
                if gitignore
                    .matched(rel, is_dir)
                    .is_ignore()
                {
                    return None;
                }

                Some(TreeNode {
                    name,
                    path,
                    is_dir,
                    children: None,
                    expanded: false,
                })
            })
            .collect(),
        Err(_) => return Vec::new(),
    };

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    entries
}
