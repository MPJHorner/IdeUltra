use std::hash::{Hash, Hasher};

/// Plain-text buffer with cheap dirty tracking via a hash of the
/// last-saved contents. Cheaper than keeping a full second copy
/// of the file in memory.
pub struct Buffer {
    pub text: String,
    saved_hash: u64,
}

impl Buffer {
    pub fn new(text: String) -> Self {
        let saved_hash = hash(&text);
        Self { text, saved_hash }
    }

    pub fn mark_clean(&mut self) {
        self.saved_hash = hash(&self.text);
    }

    pub fn is_dirty(&self) -> bool {
        hash(&self.text) != self.saved_hash
    }
}

fn hash(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
