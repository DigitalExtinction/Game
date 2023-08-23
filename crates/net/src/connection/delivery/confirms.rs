use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use async_std::channel::{SendError, Sender};

use crate::{
    header::{DatagramHeader, PackageId},
    tasks::OutDatagram,
    MAX_PACKAGE_SIZE,
};

/// The buffer is flushed after it grows beyond this number of bytes.
// Each ID is 3 bytes, thus this must be a multiple of 3.
const MAX_BUFF_SIZE: usize = 96;
/// The buffer is flushed after the oldest part is older than this.
pub(super) const MAX_BUFF_AGE: Duration = Duration::from_millis(100);

/// Buffer with datagram confirmations.
pub(super) struct ConfirmsBuffer {
    oldest: Instant,
    buffer: Vec<u8>,
    flushed: usize,
}

impl ConfirmsBuffer {
    pub(super) fn new() -> Self {
        Self {
            oldest: Instant::now(),
            buffer: Vec::with_capacity(MAX_BUFF_SIZE),
            flushed: 0,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Pushes another datagram ID to the buffer.
    pub(super) fn push(&mut self, time: Instant, id: PackageId) {
        if self.buffer.is_empty() {
            self.oldest = time;
        }
        self.buffer.extend_from_slice(&id.to_bytes());
        self.flushed = self.buffer.len();
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
    /// Next schedule time is guaranteed not to happen sooner than the returned
    /// time.
    pub(super) async fn send_confirms(
        &mut self,
        time: Instant,
        force: bool,
        addr: SocketAddr,
        datagrams: &mut Sender<OutDatagram>,
    ) -> Result<Instant, SendError<OutDatagram>> {
        let max_expiration = time + MAX_BUFF_AGE;

        let Some(expiration) = self.expiration() else {
            return Ok(max_expiration);
        };

        if force || expiration <= time || self.full() {
            while let Some(data) = self.flush(MAX_PACKAGE_SIZE) {
                datagrams
                    .send(OutDatagram::new(
                        DatagramHeader::Confirmation,
                        data.to_vec(),
                        addr,
                    ))
                    .await?;
            }

            Ok(max_expiration)
        } else {
            Ok(expiration)
        }
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
    fn test_buffer() {
        let now = Instant::now();
        let mut buf = ConfirmsBuffer::new();

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
