use std::{
    cmp::Ordering,
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use async_std::{
    channel::{SendError, Sender},
    sync::{Arc, Mutex},
};
use priority_queue::PriorityQueue;

use super::{
    book::{Connection, ConnectionBook},
    databuf::DataBuf,
};
use crate::{
    header::{DatagramHeader, DatagramId, Peers},
    tasks::dsender::OutDatagram,
};

const START_BACKOFF_MS: u64 = 220;
const MAX_TRIES: u8 = 6;

#[derive(Clone)]
pub(crate) struct Resends {
    book: Arc<Mutex<ConnectionBook<Queue>>>,
}

impl Resends {
    pub(crate) fn new() -> Self {
        Self {
            book: Arc::new(Mutex::new(ConnectionBook::new())),
        }
    }

    pub(crate) async fn sent(
        &mut self,
        time: Instant,
        addr: SocketAddr,
        id: DatagramId,
        peers: Peers,
        data: &[u8],
    ) {
        let mut book = self.book.lock().await;
        let queue = book.update(time, addr, Queue::new);
        queue.push(id, peers, data, time);
    }

    /// Processes message with datagram confirmations.
    ///
    /// The data encode IDs of delivered (and confirmed) messages so that they
    /// can be forgotten.
    pub(crate) async fn confirmed(&mut self, time: Instant, addr: SocketAddr, data: &[u8]) {
        let mut book = self.book.lock().await;
        let queue = book.update(time, addr, Queue::new);

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
        datagrams: &mut Sender<OutDatagram>,
    ) -> Result<ResendResult, SendError<OutDatagram>> {
        let mut result = ResendResult {
            failures: Vec::new(),
            next: time + Duration::from_millis(START_BACKOFF_MS),
        };

        let mut book = self.book.lock().await;

        while let Some((addr, queue)) = book.next() {
            let failure = loop {
                match queue.reschedule(buf, time) {
                    RescheduleResult::Resend { len, id, peers } => {
                        datagrams
                            .send(OutDatagram::new(
                                DatagramHeader::new_data(true, peers, id),
                                buf[..len].to_vec(),
                                addr,
                            ))
                            .await?;
                    }
                    RescheduleResult::Waiting(until) => {
                        result.next = result.next.min(until);
                        break false;
                    }
                    RescheduleResult::Empty => {
                        break false;
                    }
                    RescheduleResult::Failed => {
                        result.failures.push(addr);
                        break true;
                    }
                }
            };

            if failure {
                book.remove_current();
                result.failures.push(addr);
            }
        }

        Ok(result)
    }

    pub(crate) async fn clean(&mut self, time: Instant) {
        self.book.lock().await.clean(time);
    }
}

pub(crate) struct ResendResult {
    /// Vec of failed connections.
    pub(crate) failures: Vec<SocketAddr>,
    /// Soonest possible time of the next datagram resend.
    pub(crate) next: Instant,
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
    /// # Panics
    ///
    /// Panics if `buf` is smaller than the retrieved message.
    fn reschedule(&mut self, buf: &mut [u8], now: Instant) -> RescheduleResult {
        match self.queue.peek() {
            Some((&id, timing)) => {
                let until = timing.expiration();
                if until <= now {
                    match timing.another(now) {
                        Some(backoff) => {
                            self.queue.change_priority(&id, backoff);
                            let len = self.data.get(id, buf).unwrap();
                            let peers = *self.meta.get(&id).unwrap();
                            RescheduleResult::Resend { len, id, peers }
                        }
                        None => RescheduleResult::Failed,
                    }
                } else {
                    RescheduleResult::Waiting(until)
                }
            }
            None => RescheduleResult::Empty,
        }
    }
}

impl Connection for Queue {
    fn pending(&self) -> bool {
        !self.queue.is_empty()
    }
}

/// Rescheduling result.
pub(crate) enum RescheduleResult {
    /// A datagram is scheduled for an immediate resend.
    Resend {
        /// Length of the datagram data (written to a buffer) in bytes.
        len: usize,
        id: DatagramId,
        peers: Peers,
    },
    /// No datagram is currently scheduled for an immediate resent. This
    /// variant holds soonest possible time of a next resend.
    Waiting(Instant),
    /// There is currently no datagram scheduled for resending (immediate or
    /// future).
    Empty,
    /// A datagram expired. Id est the maximum number of resends has been
    /// reached.
    Failed,
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

    fn expiration(&self) -> Instant {
        self.expiration
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
        millis + fastrand::u64(0..millis / 2)
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
