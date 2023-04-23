use std::{collections::hash_map::Entry, num::NonZeroU32, ops::Range};

use ahash::AHashMap;
use thiserror::Error;

pub(super) struct DatagramBuffer<const ITEMS: usize, const SIZE: usize> {
    data: Vec<u8>,
    items: [Item; ITEMS],
    indices: AHashMap<NonZeroU32, usize>,
    head: usize,
    tail: usize,
    empty: bool,
}

impl<const ITEMS: usize, const SIZE: usize> DatagramBuffer<ITEMS, SIZE> {
    pub(super) fn new() -> Self {
        Self {
            data: vec![0; (ITEMS + 1) * SIZE - 1],
            items: [Item::empty(); ITEMS],
            indices: AHashMap::with_capacity(ITEMS),
            head: 0,
            tail: 0,
            empty: true,
        }
    }

    pub(super) fn push(&mut self, id: NonZeroU32, data: &[u8]) -> Result<(), BufferError> {
        if data.len() > SIZE {
            return Err(BufferError::DatagramTooLarge(data.len(), SIZE));
        }

        let next = (self.head + 1) % self.items.len();
        if self.tail == next {
            return Err(BufferError::Full);
        }

        let offset = if self.empty {
            self.empty = false;
            0
        } else {
            let prev_item = self.items[self.head];
            self.head = next;

            let offset = prev_item.offset + prev_item.size;
            if offset + data.len() > self.data.len() {
                0
            } else {
                offset
            }
        };

        match self.indices.entry(id) {
            Entry::Occupied(_) => return Err(BufferError::DuplicateId(id)),
            Entry::Vacant(entry) => entry.insert(self.head),
        };

        self.items[self.head] = Item {
            id: Some(id),
            offset,
            size: data.len(),
        };

        let capacity = self.data.len();
        self.data[self.items[self.head].range(capacity)].copy_from_slice(data);

        Ok(())
    }

    pub(super) fn get(&self, id: NonZeroU32) -> Option<&[u8]> {
        self.indices
            .get(&id)
            .map(|&index| &self.data[self.items[index].range(self.data.len())])
    }

    pub(super) fn remove(&mut self, id: NonZeroU32) {
        if let Some(index) = self.indices.remove(&id) {
            self.items[index].id = None;

            if self.tail == index {
                while self.items[self.tail].id.is_none() {
                    if self.tail == self.head {
                        self.empty = true;
                        break;
                    } else {
                        self.tail = (self.tail + 1) % self.items.len();
                    }
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub(super) enum BufferError {
    #[error("the datagram is too large: {0} > {1}")]
    DatagramTooLarge(usize, usize),
    #[error("the datagram buffer is full")]
    Full,
    #[error("duplicate datagram ID {0}")]
    DuplicateId(NonZeroU32),
}

#[derive(Clone, Copy)]
struct Item {
    id: Option<NonZeroU32>,
    offset: usize,
    size: usize,
}

impl Item {
    fn empty() -> Self {
        Self {
            id: None,
            offset: 0,
            size: 0,
        }
    }

    #[inline]
    fn range(&self, capacity: usize) -> Range<usize> {
        self.offset..(self.offset + self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buf: DatagramBuffer<3, 5> = DatagramBuffer::new();

        assert!(buf.get(NonZeroU32::new(1).unwrap()).is_none());
        // too large
        assert!(buf
            .push(
                NonZeroU32::new(1).unwrap(),
                &[100, 101, 102, 103, 104, 105, 106]
            )
            .is_err());

        let id_a = NonZeroU32::new(1).unwrap();
        buf.push(id_a, &[1, 2, 3, 4, 5]).unwrap();
        let id_b = NonZeroU32::new(2).unwrap();
        buf.push(id_b, &[6, 7, 8]).unwrap();
        let id_c = NonZeroU32::new(3).unwrap();
        buf.push(id_c, &[9, 10, 11, 12]).unwrap();
        // is full
        assert!(buf.push(NonZeroU32::new(4).unwrap(), &[13, 14]).is_err());

        assert!(buf.get(NonZeroU32::new(999).unwrap()).is_none());
        assert_eq!(buf.get(id_a).unwrap(), &[1, 2, 3, 4, 5]);
        assert_eq!(buf.get(id_b).unwrap(), &[6, 7, 8]);
        assert_eq!(buf.get(id_c).unwrap(), &[9, 10, 11, 12]);

        buf.remove(id_b);
        assert_eq!(buf.get(id_a).unwrap(), &[1, 2, 3, 4, 5]);
        assert!(buf.get(id_b).is_none());
        assert_eq!(buf.get(id_c).unwrap(), &[9, 10, 11, 12]);

        buf.remove(id_a);
        assert!(buf.get(id_a).is_none());
        assert!(buf.get(id_b).is_none());
        assert_eq!(buf.get(id_c).unwrap(), &[9, 10, 11, 12]);

        let id_d = NonZeroU32::new(5).unwrap();
        buf.push(id_d, &[201, 202, 203, 204, 205]).unwrap();
        let id_e = NonZeroU32::new(6).unwrap();
        buf.push(id_e, &[206, 207, 208, 209, 210]).unwrap();
        // is full
        assert!(buf.push(NonZeroU32::new(7).unwrap(), &[13, 14]).is_err());

        assert!(buf.get(id_a).is_none());
        assert!(buf.get(id_b).is_none());
        assert_eq!(buf.get(id_c).unwrap(), &[9, 10, 11, 12]);
        assert_eq!(buf.get(id_d).unwrap(), &[201, 202, 203, 204, 205]);
        assert_eq!(buf.get(id_e).unwrap(), &[206, 207, 208, 209, 210]);

        buf.remove(id_a);
        buf.remove(id_b);
        buf.remove(id_c);
        buf.remove(id_d);
        buf.remove(id_e);

        assert!(buf.get(id_a).is_none());
        assert!(buf.get(id_b).is_none());
        assert!(buf.get(id_c).is_none());
        assert!(buf.get(id_d).is_none());
        assert!(buf.get(id_e).is_none());

        let id_g = NonZeroU32::new(20).unwrap();
        buf.push(id_g, &[50]).unwrap();
        let id_h = NonZeroU32::new(21).unwrap();
        buf.push(id_h, &[51]).unwrap();

        assert!(buf.get(id_a).is_none());
        assert!(buf.get(id_b).is_none());
        assert!(buf.get(id_c).is_none());
        assert!(buf.get(id_d).is_none());
        assert!(buf.get(id_e).is_none());
        assert_eq!(buf.get(id_g).unwrap(), &[50]);
        assert_eq!(buf.get(id_h).unwrap(), &[51]);
    }
}
