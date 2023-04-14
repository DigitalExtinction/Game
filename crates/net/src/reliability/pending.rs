use std::{
    cmp::Ordering,
    collections::{
        hash_map::{Entry, IterMut},
        VecDeque,
    },
    iter::Peekable,
    net::SocketAddr,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use priority_queue::PriorityQueue;
use thiserror::Error;

use super::{
    buffer::DatagramBuffer,
    queue::{DatagramQueue, RescheduleError},
    types::Datagram,
};

pub(super) struct PendingRouter {
    clients: AHashMap<SocketAddr, PendingDatagrams>,
    empty: PriorityQueue<SocketAddr, Died>,
}

impl PendingRouter {
    pub(super) fn new() -> Self {
        Self {
            clients: AHashMap::new(),
            empty: PriorityQueue::new(),
        }
    }

    pub(super) fn push(&mut self, datagram: Datagram, now: Instant) -> NonZeroU32 {
        let entry = self.clients.entry(datagram.target());

        if matches!(entry, Entry::Occupied(_)) {
            self.empty.remove(&datagram.target());
        }

        let pending = entry.or_insert_with(|| PendingDatagrams::new(now));
        pending.push(datagram.data(), now)
    }

    pub(super) fn remove(&mut self, source: SocketAddr, id: NonZeroU32, now: Instant) {
        if let Some(pending) = self.clients.get_mut(&source) {
            let emptied = pending.remove(id);
            if emptied {
                self.empty.push(source, Died(now));
            }
        }
    }

    pub(super) fn cleanup(&mut self, now: Instant) {
        while self
            .empty
            .peek()
            .map(|(_, died)| (now - died.0) >= Duration::from_secs(300))
            .unwrap_or(false)
        {
            let (addr, _) = self.empty.pop().unwrap();
            self.clients.remove(&addr).unwrap();
        }
    }

    pub(super) fn reschedule(&mut self, now: Instant) -> Reschedules<'_> {
        Reschedules {
            now,
            clients: self.clients.iter_mut().peekable(),
        }
    }
}

struct Reschedules<'a> {
    now: Instant,
    clients: Peekable<IterMut<'a, SocketAddr, PendingDatagrams>>,
}

impl<'a> Iterator for Reschedules<'a> {
    type Item = Result<(NonZeroU32, Datagram<'a>), RouterError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((&addr, pending)) = self.clients.peek_mut() {
                match pending.reschedule(self.now) {
                    Ok((id, data)) => return Some(Ok((id, Datagram::new(addr, true, data)))),
                    Err(err) => match err {
                        RescheduleError::DatagramFailed(id) => {
                            return Some(Err(RouterError::DatagramFailed(addr, id)));
                        }
                        RescheduleError::None => {}
                    },
                }
            } else {
                break None;
            }

            self.clients.next();
        }
    }
}

#[derive(Error, Debug)]
pub(super) enum RouterError {
    #[error("datagram to {0} with ID {1} failed")]
    DatagramFailed(SocketAddr, NonZeroU32),
}

#[derive(Eq, PartialEq)]
struct Died(Instant);

impl Ord for Died {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for Died {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct PendingDatagrams {
    last_id: u32,
    // TODO better solve the const generics
    buffer: DatagramBuffer<1024, 512>,
    queue: DatagramQueue,
}

impl PendingDatagrams {
    fn new(now: Instant) -> Self {
        Self {
            last_id: 0,
            buffer: DatagramBuffer::new(),
            queue: DatagramQueue::new(),
        }
    }

    fn push(&mut self, data: &[u8], now: Instant) -> NonZeroU32 {
        let id = self.next_id();
        // TODO handle error
        self.buffer.push(id, data).unwrap();
        self.queue.push(id, now);
        id
    }

    fn remove(&mut self, id: NonZeroU32) -> bool {
        self.buffer.remove(id);
        self.queue.remove(id)
    }

    fn reschedule(&mut self, now: Instant) -> Result<(NonZeroU32, &[u8]), RescheduleError> {
        let id = self.queue.reschedule(now)?;
        Ok((id, self.buffer.get(id).unwrap()))
    }

    fn next_id(&mut self) -> NonZeroU32 {
        self.last_id = self.last_id.wrapping_add(1);
        if self.last_id == 0 {
            self.last_id = 1;
        }
        NonZeroU32::new(self.last_id).unwrap()
    }
}
