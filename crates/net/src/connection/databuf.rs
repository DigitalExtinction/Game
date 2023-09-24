use std::collections::VecDeque;

use ahash::AHashMap;

use crate::header::PackageId;

/// Data buffer based on a ring-buffer.
///
/// The underling data structures are optimized with the assumption that data
/// are inserted and removed in roughly FIFO manner.
pub(super) struct DataBuf {
    data: VecDeque<u8>,
    slots: VecDeque<Slot>,
    /// Mapping from datagram ID to datagram ordinal. See [`Slot::ordinal`].
    ordinals: AHashMap<PackageId, usize>,
}

impl DataBuf {
    pub(super) fn new() -> Self {
        Self {
            data: VecDeque::new(),
            slots: VecDeque::new(),
            ordinals: AHashMap::new(),
        }
    }

    /// Stores new data to the buffer.
    ///
    /// # Panics
    ///
    /// Panics if data with the same `id` is already stored.
    pub(super) fn push(&mut self, id: PackageId, data: &[u8]) {
        let (ordinal, data_offset) = match self.slots.back() {
            Some(back) => (
                back.ordinal.wrapping_add(1),
                back.data_offset.wrapping_add(back.len),
            ),
            None => (0, 0),
        };

        let prev = self.ordinals.insert(id, ordinal);
        assert!(prev.is_none());

        self.slots.push_back(Slot {
            used: true,
            ordinal,
            data_offset,
            len: data.len(),
        });

        self.data.extend(data);
    }

    /// See [`Self::get`] and [`Self::remove`].
    ///
    /// # Panics
    ///
    /// Panics if `buf` len is smaller than length of found data.
    pub(super) fn get_and_remove(&mut self, id: PackageId, buf: &mut [u8]) -> Option<usize> {
        let result = self.get(id, buf);
        if result.is_some() {
            self.remove(id);
        }
        result
    }

    /// Searches for data stored under ID `id`, and if found, writes the data
    /// to `buf` and returns length of the data.
    ///
    /// # Panics
    ///
    /// Panics if `buf` len is smaller than length of found data.
    pub(super) fn get(&self, id: PackageId, buf: &mut [u8]) -> Option<usize> {
        let Some(slot_index) = self.slot_index(id) else {
            return None;
        };

        let front = self.slots.front().unwrap();
        let slot = self.slots.get(slot_index).unwrap();

        let data_index = slot.data_offset.wrapping_sub(front.data_offset);

        assert!(buf.len() >= slot.len);

        for (source, target) in self.data.range(data_index..data_index + slot.len).zip(buf) {
            *target = *source;
        }

        Some(slot.len)
    }

    /// Removes data stored with ID `id` or does nothing if such data do not
    /// exist.
    pub(super) fn remove(&mut self, id: PackageId) {
        let Some(slot_index) = self.slot_index(id) else {
            return;
        };
        self.slots.get_mut(slot_index).unwrap().used = false;

        while let Some(front) = self.slots.front() {
            if front.used {
                break;
            }

            for _ in 0..front.len {
                self.data.pop_front();
            }
            self.slots.pop_front().unwrap();
        }
    }

    /// Get index (withing slots deque) of the slot with ID `id`.
    fn slot_index(&self, id: PackageId) -> Option<usize> {
        let Some(&ordinal) = self.ordinals.get(&id) else {
            return None;
        };
        // Slots can't be empty since the ordinal was found.
        let front = self.slots.front().unwrap();
        Some(ordinal.wrapping_sub(front.ordinal))
    }
}

/// A single slot in a bytes ring-buffer.
///
/// A slot corresponds to a single (ring-like) contiguous part of the data
/// buffer. Every byte (position) in the data buffer belongs to exactly one
/// slot (i.e. no gaps and no overlaps).
///
/// A slot may be used or unused. Unused slots are pruned once they reach end
/// of the buffer.
struct Slot {
    /// True if the slot is no longer used and may be pruned.
    used: bool,
    /// Unique number of the slot. Each new slot is assigned an ordinal, which
    /// is a wrapping increment of the ordinal of the previous slot, or 0 if
    /// there is no slots in the buffer at the time of slot creation.
    ///
    /// Given a slot `a` and the first slot in the slots buffer `f`, index of
    /// slot `a` is `a.ordinal.wrapping_sub(f.ordinal)`.
    ordinal: usize,
    /// Bytes offset of the slot since the last time the buffer was empty.
    ///
    /// Due to the fact that slots are pruned, this number may not correspond
    /// to the actual offset from the beginning of the data buffer. Offsets are
    /// modular over the maximum of `usize` (wrapping versions of arithmetic
    /// operations should be used).
    ///
    /// Wrapping difference between data offsets of two (nearby) slots
    /// corresponds to the distance of the slots inside the data buffer.
    data_offset: usize,
    /// Size (number of bytes) of the slot in the data buffer.
    len: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_buf() {
        let mut buf = [0u8; 512];
        let mut data = DataBuf::new();

        assert!(data
            .get(PackageId::try_from(1).unwrap(), &mut buf)
            .is_none());

        data.push(PackageId::try_from(12).unwrap(), &[1, 2, 3, 4, 5, 6]);
        data.push(PackageId::try_from(8).unwrap(), &[21, 22, 23]);
        assert!(data
            .get(PackageId::try_from(1).unwrap(), &mut buf)
            .is_none());
        assert_eq!(
            data.get(PackageId::try_from(8).unwrap(), &mut buf).unwrap(),
            3
        );
        assert_eq!(&buf[..3], &[21, 22, 23]);
        assert_eq!(
            data.get(PackageId::try_from(12).unwrap(), &mut buf)
                .unwrap(),
            6
        );
        assert_eq!(&buf[..6], &[1, 2, 3, 4, 5, 6]);

        for i in 100..150 {
            for j in (0..20).rev() {
                let id = PackageId::try_from(i as u32 * 100 + j as u32).unwrap();
                data.push(id, &[i, j, 23]);
            }

            for j in 0..20 {
                let id = i as u32 * 100 + j as u32;
                assert_eq!(
                    data.get(PackageId::try_from(id).unwrap(), &mut buf)
                        .unwrap(),
                    3
                );
                assert_eq!(&buf[..3], &[i, j, 23]);
                data.remove(PackageId::try_from(id).unwrap());
            }
        }
    }
}
