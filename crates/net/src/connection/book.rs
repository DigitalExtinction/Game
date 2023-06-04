use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashMap;

/// Connection info should be tossed away after this time.
const MAX_CONN_AGE: Duration = Duration::from_secs(600);

pub(super) trait Connection {
    /// Returns true if the value holds any pending actions on the connection.
    fn pending(&self) -> bool;
}

/// Bookkeeping of per connection data.
///
/// It behaves like a connection storage and a custom cyclic connection
/// "iterator".
pub(super) struct ConnectionBook<T: Connection> {
    next_index: usize,
    addrs: Vec<SocketAddr>,
    records: AHashMap<SocketAddr, ConnectionRecord<T>>,
}

impl<T: Connection> ConnectionBook<T> {
    pub(super) fn new() -> Self {
        Self {
            next_index: 0,
            addrs: Vec::new(),
            records: AHashMap::new(),
        }
    }

    /// Ensures that a connection record exists and its last update time is
    /// `time`. Mutable reference to the connection value object is returned.
    pub(super) fn update<E>(&mut self, time: Instant, addr: SocketAddr, value: E) -> &mut T
    where
        E: Fn() -> T,
    {
        let record = self
            .records
            .entry(addr)
            .and_modify(|r| r.last_update = time)
            .or_insert_with(|| {
                self.addrs.push(addr);
                ConnectionRecord {
                    last_update: time,
                    value: value(),
                }
            });

        &mut record.value
    }

    /// Forget all connections which:
    ///
    /// - has not been actively used for longer than [`MAX_CONN_AGE`],
    /// - have no pending activity.
    pub(super) fn clean(&mut self, time: Instant) {
        self.next_index = 0;
        while let Some((_addr, record)) = self.next_inner() {
            if record.is_inactive(time) {
                self.remove_current();
            }
        }
    }

    /// Yields an element (one by one) from the book. Once all elements are
    /// yielded, None is returned and the "iterator" is restarted.
    pub(super) fn next(&mut self) -> Option<(SocketAddr, &mut T)> {
        self.next_inner()
            .map(|(addr, record)| (addr, &mut record.value))
    }

    fn next_inner(&mut self) -> Option<(SocketAddr, &mut ConnectionRecord<T>)> {
        if self.next_index >= self.addrs.len() {
            self.next_index = 0;
            return None;
        }

        let addr = self.addrs[self.next_index];
        let record = self.records.get_mut(&addr).unwrap();
        self.next_index += 1;
        Some((addr, record))
    }

    /// Remove last yielded item by [`Self::next`] from the book.
    ///
    /// # Panics
    ///
    /// * Panics if [`Self::next`] yielded None when last called.
    ///
    /// * May panic if it called repeatedly between individual calls to
    ///   [`Self::next`].
    pub(super) fn remove_current(&mut self) {
        assert!(self.next_index > 0);
        self.next_index -= 1;
        let addr = self.addrs.swap_remove(self.next_index);
        self.records.remove(&addr).unwrap();
    }
}

struct ConnectionRecord<T: Connection> {
    last_update: Instant,
    value: T,
}

impl<T: Connection> ConnectionRecord<T> {
    /// Returns `true` if the connection holds no pending data and was last
    /// updated more than `max_age` in the past.
    fn is_inactive(&self, time: Instant) -> bool {
        !self.value.pending() && time - self.last_update > MAX_CONN_AGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book() {
        struct Item(u32);

        impl Connection for Item {
            fn pending(&self) -> bool {
                (self.0 % 2) == 0
            }
        }

        let mut book: ConnectionBook<Item> = ConnectionBook::new();
        assert!(book.next().is_none());

        let start = Instant::now();
        book.update(start, "1.2.3.4:1111".parse().unwrap(), || Item(1));
        book.update(start, "1.2.3.4:1112".parse().unwrap(), || Item(2));
        book.update(start, "1.2.3.4:1113".parse().unwrap(), || Item(3));
        book.update(start, "1.2.3.4:1114".parse().unwrap(), || Item(4));

        assert_eq!(book.next().unwrap().1 .0, 1);
        assert_eq!(book.next().unwrap().1 .0, 2);
        assert_eq!(book.next().unwrap().1 .0, 3);
        assert_eq!(book.next().unwrap().1 .0, 4);
        assert!(book.next().is_none());

        book.clean(start + Duration::from_millis(200));
        assert_eq!(book.next().unwrap().1 .0, 1);
        assert_eq!(book.next().unwrap().1 .0, 2);
        assert_eq!(book.next().unwrap().1 .0, 3);
        assert_eq!(book.next().unwrap().1 .0, 4);
        assert!(book.next().is_none());

        book.clean(start + MAX_CONN_AGE + Duration::from_millis(200));
        let mut numbers = vec![book.next().unwrap().1 .0, book.next().unwrap().1 .0];
        numbers.sort();
        assert_eq!(numbers, vec![2, 4]);
        assert!(book.next().is_none());
    }
}
