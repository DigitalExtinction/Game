use std::{
    cmp::Ordering,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use priority_queue::PriorityQueue;
use thiserror::Error;

pub(super) struct DatagramQueue {
    queue: PriorityQueue<NonZeroU32, Timing>,
}

impl DatagramQueue {
    pub(super) fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
        }
    }

    pub(super) fn push(&mut self, datagram: NonZeroU32, now: Instant) {
        self.queue.push(datagram, Timing::new(now));
    }

    pub(super) fn reschedule(&mut self, now: Instant) -> Result<NonZeroU32, RescheduleError> {
        match self.queue.peek() {
            Some((&id, timing)) => {
                if timing.expired(now) {
                    match timing.another(now) {
                        Some(backoff) => {
                            self.queue.change_priority(&id, backoff);
                            Ok(id)
                        }
                        None => Err(RescheduleError::DatagramFailed(id)),
                    }
                } else {
                    Err(RescheduleError::None)
                }
            }
            None => Err(RescheduleError::None),
        }
    }

    pub(super) fn remove(&mut self, datagram: NonZeroU32) {
        self.queue.remove(&datagram);
    }
}

#[derive(Error, Debug)]
pub(super) enum RescheduleError {
    #[error("datagram with ID {0} failed")]
    DatagramFailed(NonZeroU32),
    #[error("no datagram expired yet")]
    None,
}

#[derive(Eq)]
struct Timing {
    attempt: u8,
    expiration: Instant,
}

impl Timing {
    fn new(now: Instant) -> Self {
        Self {
            attempt: 1,
            expiration: Self::schedule(1, now),
        }
    }

    fn expired(&self, now: Instant) -> bool {
        self.expiration <= now
    }

    fn another(&self, now: Instant) -> Option<Self> {
        if self.attempt == 4 {
            // TODO constant
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
        // TODO constant
        100u64.pow(attempt as u32)
    }

    fn jitter(millis: u64) -> u64 {
        millis + fastrand::u64(0..millis / 2) - millis / 4
    }
}

impl Ord for Timing {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expiration
            .cmp(&other.expiration)
            .then_with(|| other.attempt.cmp(&self.attempt))
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
