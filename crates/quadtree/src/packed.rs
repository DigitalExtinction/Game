use std::ops::{Deref, DerefMut};

pub(super) struct Packed<T>(Vec<T>);

impl<T> Packed<T> {
    pub(super) fn new() -> Self {
        Self(Vec::new())
    }

    /// Removes an item from the Vec with O(1) efficiency.
    ///
    /// It can move another item to the new position as a side effect. The
    /// original index of the moved item is returned in such a case.
    pub(super) fn swap_remove(&mut self, index: usize) -> (T, Option<usize>) {
        let removed = self.0.swap_remove(index);
        let moved = if index < self.0.len() {
            Some(self.0.len())
        } else {
            None
        };
        (removed, moved)
    }
}

impl<T> Deref for Packed<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Packed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
