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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_clean() {
        let b = Buffer::new("hello".to_string());
        assert!(!b.is_dirty());
    }

    #[test]
    fn edits_make_buffer_dirty() {
        let mut b = Buffer::new("hello".to_string());
        b.text.push_str(" world");
        assert!(b.is_dirty());
    }

    #[test]
    fn mark_clean_resets_after_edit() {
        let mut b = Buffer::new("hello".to_string());
        b.text.push_str(" world");
        b.mark_clean();
        assert!(!b.is_dirty());
    }

    #[test]
    fn editing_then_undoing_returns_to_clean() {
        // A user types a character and then deletes it: hash matches saved.
        let mut b = Buffer::new("abc".to_string());
        b.text.push('x');
        assert!(b.is_dirty());
        b.text.pop();
        assert!(!b.is_dirty());
    }
}
