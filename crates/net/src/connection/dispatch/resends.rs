use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use priority_queue::PriorityQueue;

use crate::{
    connection::{book::MAX_CONN_AGE, databuf::DataBuf},
    header::{PackageHeader, PackageId},
    MAX_PACKAGE_SIZE,
};

pub(super) const START_BACKOFF_MS: u64 = 220;
const MAX_TRIES: u8 = 6;
const MAX_BASE_RESEND_INTERVAL_MS: u64 = (MAX_CONN_AGE.as_millis() / 2) as u64;

/// This struct governs reliable package re-sending (until each package is
/// confirmed).
pub(super) struct Resends {
    queue: PriorityQueue<PackageId, Timing>,
    headers: AHashMap<PackageId, PackageHeader>,
    data: DataBuf,
}

impl Resends {
    pub(super) fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            headers: AHashMap::new(),
            data: DataBuf::new(),
        }
    }

    /// Return the number of pending actions.
    pub(super) fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns true if there is no pending action.
    pub(super) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Registers new package for re-sending until it is resolved.
    ///
    /// # Panics
    ///
    /// * If the package (ID) is already stored.
    ///
    /// * If data is longer than [`MAX_PACKAGE_SIZE`].
    pub(super) fn push(&mut self, header: PackageHeader, data: &[u8], now: Instant) {
        assert!(data.len() <= MAX_PACKAGE_SIZE);
        let result = self.queue.push(header.id(), Timing::new(now));
        assert!(result.is_none());
        self.headers.insert(header.id(), header);
        self.data.push(header.id(), data);
    }

    /// Marks a package as delivered. No more re-sends will be scheduled and
    /// package data will be dropped.
    pub(super) fn resolve(&mut self, id: PackageId) {
        let result = self.queue.remove(&id);
        if result.is_some() {
            self.headers.remove(&id);
            self.data.remove(id);
        }
    }

    /// Retrieves next package to be resend or None if there is not (yet) such
    /// a package.
    ///
    /// Each package is resent multiple times with randomized exponential
    /// backoff.
    ///
    /// # Arguments
    ///
    /// * `buf` - the package payload is written to this buffer. The buffer
    ///   length must be greater or equal to the length of the payload.
    ///
    /// * `now` - current time, used for the retry scheduling.
    ///
    /// # Panics
    ///
    /// Panics if `buf` is smaller than the retrieved package payload.
    pub(super) fn reschedule(&mut self, buf: &mut [u8], now: Instant) -> RescheduleResult {
        match self.queue.peek() {
            Some((&id, timing)) => {
                let until = timing.expiration();
                if until <= now {
                    match timing.another(now) {
                        Some(backoff) => {
                            self.queue.change_priority(&id, backoff);
                            let len = self.data.get(id, buf).unwrap();
                            let header = *self.headers.get(&id).unwrap();
                            RescheduleResult::Resend { len, header }
                        }
                        None => {
                            self.queue.remove(&id).unwrap();
                            RescheduleResult::Failed
                        }
                    }
                } else {
                    RescheduleResult::Waiting(until)
                }
            }
            None => RescheduleResult::Empty,
        }
    }
}

/// Rescheduling result.
#[derive(Debug, PartialEq)]
pub(crate) enum RescheduleResult {
    /// A datagram is scheduled for an immediate resend.
    Resend {
        /// Length of the datagram data (written to a buffer) in bytes.
        len: usize,
        header: PackageHeader,
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
        let attempt = self.attempt + 1;
        if attempt == MAX_TRIES {
            None
        } else {
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
        MAX_BASE_RESEND_INTERVAL_MS.min(START_BACKOFF_MS * 2u64.pow(attempt as u32))
    }

    fn jitter(millis: u64) -> u64 {
        millis + fastrand::u64(0..millis / 2)
    }
}

impl Ord for Timing {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .expiration
            .cmp(&self.expiration)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Peers, Reliability, MAX_PACKAGE_SIZE};

    #[test]
    fn test_resends() {
        let time = Instant::now();
        let mut buf = [0u8; MAX_PACKAGE_SIZE];
        let mut resends = Resends::new();

        assert!(resends.is_empty());

        resends.push(
            PackageHeader::new(
                Reliability::Unordered,
                Peers::Server,
                PackageId::from_bytes(&[0, 0, 0]),
            ),
            &[4, 5, 8],
            time,
        );
        resends.push(
            PackageHeader::new(
                Reliability::Unordered,
                Peers::Players,
                PackageId::from_bytes(&[0, 0, 1]),
            ),
            &[4, 5, 8, 9],
            time + Duration::from_millis(10_010),
        );
        resends.push(
            PackageHeader::new(
                Reliability::Unordered,
                Peers::Server,
                PackageId::from_bytes(&[0, 0, 2]),
            ),
            &[4, 5, 8, 9, 10],
            time + Duration::from_millis(50_020),
        );
        assert_eq!(resends.len(), 3);

        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(20)),
            RescheduleResult::Resend {
                len: 3,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 0]),
                )
            }
        );
        assert_eq!(&buf[..3], &[4, 5, 8]);
        resends.resolve(PackageId::from_bytes(&[0, 0, 0]));

        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(20)),
            RescheduleResult::Resend {
                len: 4,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Players,
                    PackageId::from_bytes(&[0, 0, 1])
                )
            }
        );
        assert_eq!(&buf[..4], &[4, 5, 8, 9]);
        resends.resolve(PackageId::from_bytes(&[0, 0, 1]));

        assert!(matches!(
            resends.reschedule(&mut buf, time + Duration::from_secs(20)),
            RescheduleResult::Waiting(_)
        ));

        // 1st resend
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(1000)),
            RescheduleResult::Resend {
                len: 5,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 2])
                )
            }
        );
        // 2nd resend
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(2000)),
            RescheduleResult::Resend {
                len: 5,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 2])
                )
            }
        );
        // 3rd resend
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(3000)),
            RescheduleResult::Resend {
                len: 5,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 2])
                )
            }
        );
        // 4th resend
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(4000)),
            RescheduleResult::Resend {
                len: 5,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 2])
                )
            }
        );
        // 5th resend
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(5000)),
            RescheduleResult::Resend {
                len: 5,
                header: PackageHeader::new(
                    Reliability::Unordered,
                    Peers::Server,
                    PackageId::from_bytes(&[0, 0, 2])
                )
            }
        );
        // 6th resend (7th try) => failure
        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(6000)),
            RescheduleResult::Failed
        );

        assert_eq!(
            resends.reschedule(&mut buf, time + Duration::from_secs(7000)),
            RescheduleResult::Empty
        );
    }

    #[test]
    fn test_timing() {
        let time = Instant::now();
        let first = Timing::new(time);
        let second = Timing::new(time + Duration::from_secs(3600));
        assert_eq!(first.cmp(&second), Ordering::Greater);
    }
}
