use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use priority_queue::PriorityQueue;
use thiserror::Error;

use crate::{
    databuf::DataBuf,
    header::{DatagramId, Peers},
};

const START_BACKOFF_MS: u64 = 220;
const MAX_TRIES: u8 = 6;

/// This struct governs reliable message re-sending (until each message is
/// confirmed).
pub(crate) struct ResendQueue {
    queue: PriorityQueue<DatagramId, Timing>,
    meta: AHashMap<DatagramId, Peers>,
    data: DataBuf,
}

impl ResendQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            meta: AHashMap::new(),
            data: DataBuf::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Registers new message for re-sending until it is resolved.
    pub(crate) fn push(&mut self, id: DatagramId, peers: Peers, data: &[u8], now: Instant) {
        self.queue.push(id, Timing::new(now));
        self.meta.insert(id, peers);
        self.data.push(id, data);
    }

    /// Marks a message as delivered. No more re-sends will be scheduled and
    /// message data will be dropped.
    pub(crate) fn resolve(&mut self, id: DatagramId) {
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
    pub(crate) fn reschedule(
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

#[derive(Error, Debug)]
pub(crate) enum RescheduleError {
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
