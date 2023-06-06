use std::{
    cmp::Ordering,
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use priority_queue::PriorityQueue;
use thiserror::Error;

use super::{
    book::{Connection, ConnectionBook},
    databuf::DataBuf,
};
use crate::{
    header::{DatagramHeader, DatagramId, Peers, HEADER_SIZE},
    messages::Messages,
    SendError,
};

const START_BACKOFF_MS: u64 = 220;
const MAX_TRIES: u8 = 6;

pub(crate) struct Resends {
    book: ConnectionBook<Queue>,
}

impl Resends {
    pub(crate) fn new() -> Self {
        Self {
            book: ConnectionBook::new(),
        }
    }

    pub(crate) fn sent(
        &mut self,
        time: Instant,
        addr: SocketAddr,
        id: DatagramId,
        peers: Peers,
        data: &[u8],
    ) {
        let queue = self.book.update(time, addr, Queue::new);
        queue.push(id, peers, data, time);
    }

    /// Processes message with datagram confirmations.
    ///
    /// The data encode IDs of delivered (and confirmed) messages so that they
    /// can be forgotten.
    pub(crate) fn confirmed(&mut self, time: Instant, addr: SocketAddr, data: &[u8]) {
        let queue = self.book.update(time, addr, Queue::new);

        for i in 0..data.len() / 3 {
            let offset = i * 4;
            let id = DatagramId::from_bytes(&data[offset..offset + 3]);
            queue.resolve(id);
        }
    }

    /// Re-send all messages already due for re-sending.
    pub(crate) async fn resend(
        &mut self,
        time: Instant,
        buf: &mut [u8],
        messages: &mut Messages,
    ) -> Result<Vec<SocketAddr>, SendError> {
        let mut failures = Vec::new();

        while let Some((addr, queue)) = self.book.next() {
            let failure = loop {
                match queue.reschedule(&mut buf[HEADER_SIZE..], time) {
                    Ok(Some((len, id, peers))) => {
                        messages
                            .send(
                                &mut buf[..len + HEADER_SIZE],
                                DatagramHeader::new_data(true, peers, id),
                                addr,
                            )
                            .await?;
                    }
                    Ok(None) => break false,
                    Err(_) => {
                        failures.push(addr);
                        break true;
                    }
                }
            };

            if failure {
                self.book.remove_current();
                failures.push(addr);
            }
        }

        Ok(failures)
    }

    pub(crate) fn clean(&mut self, time: Instant) {
        self.book.clean(time);
    }
}

/// This struct governs reliable message re-sending (until each message is
/// confirmed).
struct Queue {
    queue: PriorityQueue<DatagramId, Timing>,
    meta: AHashMap<DatagramId, Peers>,
    data: DataBuf,
}

impl Queue {
    fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            meta: AHashMap::new(),
            data: DataBuf::new(),
        }
    }

    /// Registers new message for re-sending until it is resolved.
    fn push(&mut self, id: DatagramId, peers: Peers, data: &[u8], now: Instant) {
        self.queue.push(id, Timing::new(now));
        self.meta.insert(id, peers);
        self.data.push(id, data);
    }

    /// Marks a message as delivered. No more re-sends will be scheduled and
    /// message data will be dropped.
    fn resolve(&mut self, id: DatagramId) {
        let result = self.queue.remove(&id);
        if result.is_some() {
            self.meta.remove(&id);
            self.data.remove(id);
        }
    }

    /// Retrieves next message to be resend or None if there is not (yet) such
    /// a message.
    ///
    /// Each message is resent multiple times with randomized exponential
    /// backoff.
    ///
    /// # Arguments
    ///
    /// * `buf` - the message data is written to this buffer. The buffer length
    ///   must be greater or equal to the length of the message.
    ///
    /// * `now` - current time, used for the retry scheduling.
    ///
    /// # Returns
    ///
    /// Returns a tuple with number of bytes of retrieved data and header of
    /// the retrieved message.
    ///
    /// # Panics
    ///
    /// Panics if `buf` is smaller than the retrieved message.
    fn reschedule(
        &mut self,
        buf: &mut [u8],
        now: Instant,
    ) -> Result<Option<(usize, DatagramId, Peers)>, RescheduleError> {
        match self.queue.peek() {
            Some((&id, timing)) => {
                if timing.expired(now) {
                    match timing.another(now) {
                        Some(backoff) => {
                            self.queue.change_priority(&id, backoff);
                            let len = self.data.get(id, buf).unwrap();
                            let peers = *self.meta.get(&id).unwrap();
                            Ok(Some((len, id, peers)))
                        }
                        None => Err(RescheduleError::DatagramFailed(id)),
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl Connection for Queue {
    fn pending(&self) -> bool {
        !self.queue.is_empty()
    }
}

#[derive(Error, Debug)]
pub(super) enum RescheduleError {
    #[error("datagram {0} failed")]
    DatagramFailed(DatagramId),
}

#[derive(Eq)]
struct Timing {
    attempt: u8,
    expiration: Instant,
}

impl Timing {
    fn new(now: Instant) -> Self {
        Self {
            attempt: 0,
            expiration: Self::schedule(0, now),
        }
    }

    fn expired(&self, now: Instant) -> bool {
        self.expiration <= now
    }

    fn another(&self, now: Instant) -> Option<Self> {
        if self.attempt == MAX_TRIES {
            None
        } else {
            let attempt = self.attempt + 1;
            Some(Self {
                attempt,
                expiration: Self::schedule(attempt, now),
            })
        }
    }

    fn schedule(attempt: u8, now: Instant) -> Instant {
        let millis = Self::jitter(Self::backoff(attempt));
        now + Duration::from_millis(millis)
    }

    fn backoff(attempt: u8) -> u64 {
        START_BACKOFF_MS * 2u64.pow(attempt as u32)
    }

    fn jitter(millis: u64) -> u64 {
        millis + fastrand::u64(0..millis / 2) - millis / 4
    }
}

impl Ord for Timing {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expiration
            .cmp(&other.expiration)
            .then_with(|| self.attempt.cmp(&other.attempt))
    }
}

impl PartialOrd for Timing {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timing {
    fn eq(&self, other: &Self) -> bool {
        self.expiration == other.expiration && self.attempt == other.attempt
    }
}
