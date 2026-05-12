pub mod buffer;
pub mod language;
pub mod position;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syntect::parsing::SyntaxReference;

use self::buffer::Buffer;
use self::language::{syntax_for_path, SYNTAX_SET};
use crate::ui::highlight::HighlightCache;

pub struct EditorTab {
    pub path: PathBuf,
    pub buffer: Buffer,
    pub display_name: String,
    pub syntax_name: String,
    pub highlight: HighlightCache,
    /// Set when the watcher saw the file change while we had unsaved
    /// edits, OR when we couldn't reload it automatically.
    pub external_change: bool,
    /// Hash of the last contents we wrote to the crash-recovery store.
    /// Lets the app skip the write when nothing has changed since the
    /// last recovery snapshot — cheap dirty-check.
    pub last_recovered_hash: Option<u64>,
    /// True when the tab hasn't been saved yet. `self.path` is a
    /// placeholder like `Untitled 1` and is NOT a real filesystem path.
    /// Save / autosave skip this tab until the user runs Save As.
    pub is_untitled: bool,
}

impl EditorTab {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let display_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        let syntax = syntax_for_path(&path);
        Ok(Self {
            buffer: Buffer::new(text),
            path,
            display_name,
            syntax_name: syntax.name.clone(),
            highlight: HighlightCache::default(),
            external_change: false,
            last_recovered_hash: None,
            is_untitled: false,
        })
    }

    /// Brand-new in-memory buffer with no on-disk path yet. `n` is a
    /// monotonically increasing counter so the tab labels read
    /// `Untitled 1`, `Untitled 2`, etc.
    pub fn new_untitled(n: usize) -> Self {
        let label = format!("Untitled {n}");
        let placeholder = PathBuf::from(&label);
        Self {
            buffer: Buffer::new(String::new()),
            path: placeholder,
            display_name: label,
            syntax_name: SYNTAX_SET.find_syntax_plain_text().name.clone(),
            highlight: HighlightCache::default(),
            external_change: false,
            last_recovered_hash: None,
            is_untitled: true,
        }
    }

    /// Promote an untitled tab to a real on-disk file. Updates path,
    /// display name, syntax, then writes through.
    pub fn save_as(&mut self, new_path: impl AsRef<Path>) -> Result<()> {
        let new_path = new_path.as_ref().to_path_buf();
        self.path = new_path;
        self.display_name = self
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        self.syntax_name = syntax_for_path(&self.path).name.clone();
        self.is_untitled = false;
        self.save()
    }

    /// Build a tab from a path plus contents already in memory.
    /// Used by the crash-recovery flow.
    pub fn from_recovered(path: PathBuf, recovered_text: String) -> Self {
        let display_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        let syntax = syntax_for_path(&path);
        let mut buf = Buffer::new(String::new());
        // Treat the recovered text as "edited since saved" so the dirty dot
        // appears and the user sees they should save.
        buf.text = recovered_text;
        Self {
            buffer: buf,
            path,
            display_name,
            syntax_name: syntax.name.clone(),
            highlight: HighlightCache::default(),
            external_change: false,
            last_recovered_hash: None,
            is_untitled: false,
        }
    }

    /// Re-read the file from disk into the buffer. Clears `external_change`
    /// and marks the buffer clean. Caller should keep the user's caret
    /// position if they care; this method does not touch egui state.
    pub fn reload_from_disk(&mut self) -> Result<()> {
        let text = std::fs::read_to_string(&self.path)
            .with_context(|| format!("read {}", self.path.display()))?;
        self.buffer = Buffer::new(text);
        self.external_change = false;
        Ok(())
    }

    /// Save the buffer to disk. The caller decides whether to normalize
    /// the contents first via `apply_save_normalization`. Returns an
    /// error for untitled tabs — the caller should route those through
    /// `save_as` after prompting for a path.
    pub fn save(&mut self) -> Result<()> {
        if self.is_untitled {
            anyhow::bail!("untitled buffer — use Save As");
        }
        std::fs::write(&self.path, self.buffer.text.as_bytes())
            .with_context(|| format!("write {}", self.path.display()))?;
        self.buffer.mark_clean();
        Ok(())
    }

    /// Apply on-save text normalisation in-place. Pure delegations to
    /// `crate::normalize` so this can be tested without touching disk.
    pub fn apply_save_normalization(
        &mut self,
        trim_whitespace: bool,
        ensure_final_newline: bool,
    ) {
        if trim_whitespace {
            self.buffer.text = crate::normalize::trim_trailing_whitespace(&self.buffer.text);
        }
        if ensure_final_newline {
            self.buffer.text = crate::normalize::ensure_final_newline(&self.buffer.text);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.is_dirty()
    }

    pub fn syntax(&self) -> &'static SyntaxReference {
        SYNTAX_SET
            .find_syntax_by_name(&self.syntax_name)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    }
}
