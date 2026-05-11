//! Tab MRU (most-recently-used) tracking for the quick switcher.
//!
//! Pure list operations over indices into the tab vector. Whenever a
//! tab is activated, we move that index to the head; when a tab is
//! closed, we drop the index and shift larger indices down by 1.

#[derive(Debug, Clone, Default)]
pub struct TabMru {
    /// Tab indices, most-recently-used first. Always equal length to
    /// the tab vector (the app keeps them in sync via `touch` / `removed`).
    pub order: Vec<usize>,
}

impl TabMru {
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Mark `tab_idx` as just-used. Inserts at the head; if it was
    /// already present, moves it to the head.
    pub fn touch(&mut self, tab_idx: usize) {
        self.order.retain(|i| *i != tab_idx);
        self.order.insert(0, tab_idx);
    }

    /// Drop the tab at `tab_idx` and rebase larger indices.
    pub fn removed(&mut self, tab_idx: usize) {
        self.order.retain(|i| *i != tab_idx);
        for entry in self.order.iter_mut() {
            if *entry > tab_idx {
                *entry -= 1;
            }
        }
    }

    /// Walk `step` positions forward (1) or backward (-1) through the
    /// MRU list, wrapping at the ends. Returns the resulting index, or
    /// `None` if the list is empty.
    pub fn cycle(&self, current_pos: usize, step: i32) -> Option<usize> {
        if self.order.is_empty() {
            return None;
        }
        let n = self.order.len() as i32;
        let next = ((current_pos as i32) + step).rem_euclid(n);
        Some(next as usize)
    }

    pub fn tab_at(&self, mru_pos: usize) -> Option<usize> {
        self.order.get(mru_pos).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_inserts_at_head() {
        let mut m = TabMru::default();
        m.touch(2);
        m.touch(0);
        m.touch(1);
        assert_eq!(m.order, vec![1, 0, 2]);
    }

    #[test]
    fn touch_existing_moves_to_head_no_duplicate() {
        let mut m = TabMru::default();
        m.touch(0);
        m.touch(1);
        m.touch(2);
        m.touch(0); // revisit
        assert_eq!(m.order, vec![0, 2, 1]);
    }

    #[test]
    fn removed_drops_and_rebases_larger() {
        let mut m = TabMru { order: vec![0, 2, 4, 1, 3] };
        m.removed(2);
        // Indices > 2 shift down by 1; the entry `2` itself is dropped.
        assert_eq!(m.order, vec![0, 3, 1, 2]);
    }

    #[test]
    fn removed_index_not_present_still_rebases() {
        let mut m = TabMru { order: vec![1, 3] };
        m.removed(0);
        assert_eq!(m.order, vec![0, 2]);
    }

    #[test]
    fn cycle_wraps_at_ends_in_both_directions() {
        let m = TabMru { order: vec![10, 20, 30] };
        assert_eq!(m.cycle(0, 1), Some(1));
        assert_eq!(m.cycle(2, 1), Some(0));
        assert_eq!(m.cycle(0, -1), Some(2));
        assert_eq!(m.cycle(1, -1), Some(0));
    }

    #[test]
    fn cycle_on_empty_returns_none() {
        let m = TabMru::default();
        assert_eq!(m.cycle(0, 1), None);
    }

    #[test]
    fn tab_at_indexes_into_mru_order() {
        let m = TabMru { order: vec![5, 7, 9] };
        assert_eq!(m.tab_at(0), Some(5));
        assert_eq!(m.tab_at(2), Some(9));
        assert_eq!(m.tab_at(99), None);
    }
}
