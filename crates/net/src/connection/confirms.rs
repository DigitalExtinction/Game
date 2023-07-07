use std::{
    cmp::Ordering,
    net::SocketAddr,
    time::{Duration, Instant},
};

use ahash::AHashSet;
use async_std::{
    channel::{SendError, Sender},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use super::book::{Connection, ConnectionBook};
use crate::{
    header::{DatagramHeader, PackageId, PackageIdRange},
    protocol::MAX_PACKAGE_SIZE,
    tasks::OutDatagram,
};

/// The buffer is flushed after it grows beyond this number of bytes.
// Each ID is 3 bytes, thus this must be a multiple of 3.
const MAX_BUFF_SIZE: usize = 96;
/// The buffer is flushed after the oldest part is older than this.
const MAX_BUFF_AGE: Duration = Duration::from_millis(100);
const MAX_SKIPPED: usize = 1024;

#[derive(Clone)]
pub(crate) struct Confirmations {
    book: Arc<Mutex<ConnectionBook<IdReceiver>>>,
}

impl Confirmations {
    pub(crate) fn new() -> Self {
        Self {
            book: Arc::new(Mutex::new(ConnectionBook::new())),
        }
    }

    /// This method checks whether a package with `id` from `addr` was already
    /// marked as received in the past. If so it returns true. Otherwise, it
    /// marks the package as received and returns false.
    ///
    /// This method should be called exactly once after each reliable package
    /// is delivered and in order.
    pub(crate) async fn received(
        &mut self,
        time: Instant,
        addr: SocketAddr,
        id: PackageId,
    ) -> Result<bool, PackageIdError> {
        self.book
            .lock()
            .await
            .update(time, addr, IdReceiver::new)
            .push(time, id)
    }

    /// Send package confirmation datagrams.
    ///
    /// Not all confirmations are sent because there is a small delay to enable
    /// grouping.
    ///
    /// # Arguments
    ///
    /// * `time` - current time.
    ///
    /// * `force` - if true, all pending confirmations will be sent.
    ///
    /// * `datagrams` - output datagrams with the confirmations will be send to
    ///   this channel.
    ///
    /// # Returns
    ///
    /// On success, it returns an estimation of the next resend schedule time.
    pub(crate) async fn send_confirms(
        &mut self,
        time: Instant,
        force: bool,
        datagrams: &mut Sender<OutDatagram>,
    ) -> Result<Instant, SendError<OutDatagram>> {
        let mut next = Instant::now() + MAX_BUFF_AGE;
        let mut book = self.book.lock().await;

        while let Some((addr, id_receiver)) = book.next() {
            if let Some(expiration) = id_receiver.buffer.expiration() {
                if force || expiration <= time || id_receiver.buffer.full() {
                    while let Some(data) = id_receiver.buffer.flush(MAX_PACKAGE_SIZE) {
                        datagrams
                            .send(OutDatagram::new(
                                DatagramHeader::Confirmation,
                                data.to_vec(),
                                addr,
                            ))
                            .await?;
                    }
                } else {
                    next = next.min(expiration);
                }
            }
        }

        Ok(next)
    }

    pub(crate) async fn clean(&mut self, time: Instant) {
        self.book.lock().await.clean(time);
    }
}

struct IdReceiver {
    duplicates: Duplicates,
    buffer: Buffer,
}

impl IdReceiver {
    fn new() -> Self {
        Self {
            duplicates: Duplicates::new(),
            buffer: Buffer::new(),
        }
    }

    /// Registers a package as received and returns whether the it was a
    /// duplicate delivery.
    fn push(&mut self, time: Instant, id: PackageId) -> Result<bool, PackageIdError> {
        // Push to the buffer unconditionally, because the reason behind the
        // re-delivery might be loss of the confirmation datagram.
        self.buffer.push(time, id);
        self.duplicates.process(id)
    }
}

impl Connection for IdReceiver {
    fn pending(&self) -> bool {
        !self.buffer.is_empty()
    }
}

struct Duplicates {
    highest_id: Option<PackageId>,
    holes: AHashSet<PackageId>,
}

impl Duplicates {
    fn new() -> Self {
        Self {
            highest_id: None,
            holes: AHashSet::new(),
        }
    }

    /// Registers package as delivered and returns true if it was already
    /// delivered in the past.
    fn process(&mut self, id: PackageId) -> Result<bool, PackageIdError> {
        let range_start = match self.highest_id {
            Some(highest) => match highest.ordering(id) {
                Ordering::Less => highest.incremented(),
                Ordering::Greater => {
                    return Ok(!self.holes.remove(&id));
                }
                Ordering::Equal => {
                    return Ok(true);
                }
            },
            None => PackageId::zero(),
        };

        let range = PackageIdRange::range(range_start, id);
        let skipped = range.size_hint().1.unwrap() + self.holes.len();
        if skipped > MAX_SKIPPED {
            return Err(PackageIdError::TooManySkipped(skipped));
        }

        self.highest_id = Some(id);
        for hole in range {
            self.holes.insert(hole);
        }

        Ok(false)
    }
}

#[derive(Error, Debug)]
pub(crate) enum PackageIdError {
    #[error("Too many packages skipped: {0}")]
    TooManySkipped(usize),
}

/// Buffer with datagram confirmations.
struct Buffer {
    oldest: Instant,
    buffer: Vec<u8>,
    flushed: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            oldest: Instant::now(),
            buffer: Vec::with_capacity(MAX_BUFF_SIZE),
            flushed: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Pushes another datagram ID to the buffer.
    fn push(&mut self, time: Instant, id: PackageId) {
        if self.buffer.is_empty() {
            self.oldest = time;
        }
        self.buffer.extend_from_slice(&id.to_bytes());
        self.flushed = self.buffer.len();
    }

    /// Returns time when the buffer expires, i.e. time when it becomes
    /// necessary to flush the buffer and send the confirmations.
    fn expiration(&self) -> Option<Instant> {
        if self.flushed == 0 {
            None
        } else {
            Some(self.oldest + MAX_BUFF_AGE)
        }
    }

    fn full(&self) -> bool {
        self.buffer.len() >= MAX_BUFF_SIZE
    }

    /// Return accumulated bytes from the buffer if it is not empty. The number
    /// of returned bytes is always smaller than `max_size`. This method should
    /// be called repeatedly until it returns None.
    fn flush(&mut self, max_size: usize) -> Option<&[u8]> {
        self.buffer.truncate(self.flushed);

        if self.buffer.is_empty() {
            None
        } else {
            // Make sure it is multiple of 4 (i.e. larges multiple of 4 smaller
            // or equal than the original).
            let size = self.buffer.len().min(max_size & (usize::MAX - 3));
            self.flushed = self.buffer.len() - size;
            Some(&self.buffer[self.flushed..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicates() {
        let mut duplicates = Duplicates::new();

        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 2]))
            .unwrap());
        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 1]))
            .unwrap());
        assert!(duplicates
            .process(PackageId::from_bytes(&[0, 0, 1]))
            .unwrap());
        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 0]))
            .unwrap());

        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 5]))
            .unwrap());
        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 3]))
            .unwrap());
        assert!(duplicates
            .process(PackageId::from_bytes(&[0, 0, 5]))
            .unwrap());
        assert!(!duplicates
            .process(PackageId::from_bytes(&[0, 0, 6]))
            .unwrap());
        assert!(duplicates
            .process(PackageId::from_bytes(&[0, 0, 3]))
            .unwrap());

        assert!(matches!(
            duplicates.process(PackageId::from_bytes(&[50, 0, 6])),
            Err(PackageIdError::TooManySkipped(3276800))
        ));
    }

    #[test]
    fn test_buffer() {
        let now = Instant::now();
        let mut buf = Buffer::new();

        assert!(buf.flush(13).is_none());
        assert!(buf.expiration().is_none());
        assert!(!buf.full());

        buf.push(now, 1042.try_into().unwrap());
        assert!(buf.expiration().unwrap() > now);
        assert!(!buf.full());

        assert_eq!(buf.flush(13).unwrap(), &[0, 4, 18]);
        assert!(buf.expiration().is_none());
        assert!(!buf.full());

        assert!(buf.flush(13).is_none());
        assert!(buf.expiration().is_none());
        assert!(!buf.full());

        buf.push(now, 43.try_into().unwrap());
        assert_eq!(buf.expiration(), Some(now + MAX_BUFF_AGE));
        assert!(!buf.full());

        assert_eq!(buf.flush(13).unwrap(), &[0, 0, 43]);
        assert!(buf.flush(13).is_none());
        assert!(!buf.full());

        for i in 0..32 {
            buf.push(now, (100 + i).try_into().unwrap());

            if i < 31 {
                assert!(!buf.full());
            } else {
                assert!(buf.full());
            }
        }

        for i in 0..8 {
            assert_eq!(
                buf.flush(12 + (i as usize) % 3).unwrap(),
                &[
                    0,
                    0,
                    128 - i * 4,
                    0,
                    0,
                    129 - i * 4,
                    0,
                    0,
                    130 - i * 4,
                    0,
                    0,
                    131 - i * 4
                ]
            );
        }

        assert!(buf.flush(8).is_none());
    }
}
