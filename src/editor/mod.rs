pub mod buffer;
pub mod language;

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
        })
    }

    pub fn save(&mut self) -> Result<()> {
        std::fs::write(&self.path, self.buffer.text.as_bytes())
            .with_context(|| format!("write {}", self.path.display()))?;
        self.buffer.mark_clean();
        Ok(())
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
