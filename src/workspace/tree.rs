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

    /// Invalidate cached children for every directory that contains, or
    /// is, `affected_path`. Subsequent expansion will re-read from disk.
    /// Returns the number of nodes invalidated.
    pub fn invalidate_containing(&mut self, affected_path: &Path) -> usize {
        let workspace_root = self.workspace_root.clone();
        invalidate_recursive(&mut self.root, &workspace_root, affected_path)
    }
}

fn invalidate_recursive(node: &mut TreeNode, workspace_root: &Path, affected: &Path) -> usize {
    let mut n = 0;
    if !node.is_dir {
        return 0;
    }
    // Only descend if the affected path is under this directory.
    if !path_is_under(affected, &node.path) && node.path != *affected {
        return 0;
    }
    if node.children.is_some() {
        // Invalidate children first so the count is meaningful.
        if let Some(children) = node.children.as_mut() {
            for child in children.iter_mut() {
                n += invalidate_recursive(child, workspace_root, affected);
            }
        }
        node.children = None;
        n += 1;
    }
    n
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn names(tree: &FileTree) -> Vec<&str> {
        tree.root
            .children
            .as_ref()
            .map(|cs| cs.iter().map(|c| c.name.as_str()).collect())
            .unwrap_or_default()
    }

    #[test]
    fn root_loads_one_level_eagerly() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("nested.txt"), "").unwrap();

        let tree = FileTree::new(dir.path()).unwrap();
        assert_eq!(names(&tree), vec!["sub", "a.txt"]);
        // Sub directory's own children are NOT loaded yet (lazy).
        let sub = tree.root.children.as_ref().unwrap().iter().find(|c| c.name == "sub").unwrap();
        assert!(sub.children.is_none(), "subdir children must be lazy");
    }

    #[test]
    fn directories_sort_before_files_case_insensitive() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Zzz.txt"), "").unwrap();
        fs::write(dir.path().join("aaa.txt"), "").unwrap();
        fs::create_dir(dir.path().join("Mid")).unwrap();
        fs::create_dir(dir.path().join("alpha")).unwrap();

        let tree = FileTree::new(dir.path()).unwrap();
        // Dirs first (alpha < Mid case-insensitively), then files (aaa < Zzz).
        assert_eq!(names(&tree), vec!["alpha", "Mid", "aaa.txt", "Zzz.txt"]);
    }

    #[test]
    fn dotgit_is_always_hidden() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();

        let tree = FileTree::new(dir.path()).unwrap();
        assert_eq!(names(&tree), vec!["README.md"]);
    }

    #[test]
    fn gitignored_entries_are_filtered() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "target/\n*.log\n").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("debug.log"), "").unwrap();
        fs::write(dir.path().join("src.rs"), "").unwrap();

        let tree = FileTree::new(dir.path()).unwrap();
        // .gitignore itself isn't ignored — it's still visible.
        assert_eq!(names(&tree), vec![".gitignore", "src.rs"]);
    }

    #[test]
    fn invalidate_containing_clears_loaded_children() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("a.txt"), "").unwrap();

        let mut tree = FileTree::new(dir.path()).unwrap();
        tree.load_children(&[0]); // load sub
        assert!(tree.root.children.as_ref().unwrap()[0].children.is_some());

        let touched = dir.path().join("sub").join("a.txt");
        let n = tree.invalidate_containing(&touched);
        assert!(n >= 1, "should invalidate at least the root + sub");
        // After invalidation, children are reset and will lazy-load on next expand.
        assert!(tree.root.children.is_none());
    }

    #[test]
    fn lazy_load_children_is_idempotent() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("a.txt"), "").unwrap();

        let mut tree = FileTree::new(dir.path()).unwrap();
        tree.load_children(&[0]); // sub
        let len_first = tree.root.children.as_ref().unwrap()[0]
            .children
            .as_ref()
            .unwrap()
            .len();
        tree.load_children(&[0]); // again — should not re-walk
        let len_second = tree.root.children.as_ref().unwrap()[0]
            .children
            .as_ref()
            .unwrap()
            .len();
        assert_eq!(len_first, len_second);
        assert_eq!(len_first, 1);
    }
}
