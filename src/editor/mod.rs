pub mod buffer;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use self::buffer::Buffer;

pub struct EditorTab {
    pub path: PathBuf,
    pub buffer: Buffer,
    pub display_name: String,
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
        Ok(Self {
            buffer: Buffer::new(text),
            path,
            display_name,
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
}
