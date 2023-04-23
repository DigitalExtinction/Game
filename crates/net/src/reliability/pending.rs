use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, hash_set::Iter},
    net::SocketAddr,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use ahash::{AHashMap, AHashSet};
use priority_queue::PriorityQueue;
use thiserror::Error;

use super::{
    buffer::DatagramBuffer,
    queue::{DatagramQueue, RescheduleError},
    types::Datagram,
};

pub(super) struct PendingRouter {
    clients: AHashSet<SocketAddr>,
    pending: AHashMap<SocketAddr, PendingDatagrams>,
    empty: PriorityQueue<SocketAddr, Died>,
}

impl PendingRouter {
    pub(super) fn new() -> Self {
        Self {
            clients: AHashSet::new(),
            pending: AHashMap::new(),
            empty: PriorityQueue::new(),
        }
    }

    pub(super) fn push(&mut self, datagram: Datagram, now: Instant) -> NonZeroU32 {
        let entry = self.pending.entry(datagram.target());

        let pending = match entry {
            Entry::Occupied(entry) => {
                self.empty.remove(&datagram.target());
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                self.clients.insert(datagram.target());
                entry.insert(PendingDatagrams::new(now))
            }
        };

        pending.push(datagram.data(), now)
    }

    pub(super) fn remove(&mut self, source: SocketAddr, id: NonZeroU32, now: Instant) {
        if let Some(pending) = self.pending.get_mut(&source) {
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
            self.pending.remove(&addr).unwrap();
            let removed = self.clients.remove(&addr);
            debug_assert!(removed);
        }
    }

    pub(super) fn reschedule(&mut self, now: Instant) -> Reschedules<'_> {
        Reschedules::new(self.clients.iter(), &mut self.pending, now)
    }
}

pub(super) struct Reschedules<'a> {
    now: Instant,
    current: Option<SocketAddr>,
    clients: Iter<'a, SocketAddr>,
    pending: &'a mut AHashMap<SocketAddr, PendingDatagrams>,
}

impl<'a> Reschedules<'a> {
    fn new(
        clients: Iter<'a, SocketAddr>,
        pending: &'a mut AHashMap<SocketAddr, PendingDatagrams>,
        now: Instant,
    ) -> Self {
        Self {
            now,
            current: None,
            clients,
            pending,
        }
    }

    pub(super) fn next<'b>(
        &'b mut self,
    ) -> Option<Result<(NonZeroU32, Datagram<'b>), RouterError>> {
        loop {
            if self.current.is_none() {
                self.current = self.clients.next().copied();
            }
            let Some(addr) = self.current else { return None };
            let rescheduled = self.pending.get_mut(&addr).unwrap().reschedule(self.now);

            match rescheduled {
                Ok(id) => {
                    let data = self.pending.get(&addr).unwrap().data(id).unwrap();
                    return Some(Ok((id, Datagram::new(addr, true, data))));
                }
                Err(err) => match err {
                    RescheduleError::DatagramFailed(id) => {
                        self.current = None;
                        return Some(Err(RouterError::DatagramFailed(addr, id)));
                    }
                    RescheduleError::None => {
                        self.current = None;
                    }
                },
            }
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

    fn reschedule(&mut self, now: Instant) -> Result<NonZeroU32, RescheduleError> {
        self.queue.reschedule(now)
    }

    fn data(&self, id: NonZeroU32) -> Option<&[u8]> {
        self.buffer.get(id)
    }

    fn next_id(&mut self) -> NonZeroU32 {
        self.last_id = self.last_id.wrapping_add(1);
        if self.last_id == 0 {
            self.last_id = 1;
        }
        NonZeroU32::new(self.last_id).unwrap()
    }
}
