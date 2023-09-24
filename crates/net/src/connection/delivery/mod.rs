use std::{net::SocketAddr, time::Instant};

use async_std::{
    channel::{SendError, Sender},
    sync::{Arc, Mutex, MutexGuard},
};

pub(crate) use self::received::ReceivedIdError;
use self::{
    confirms::{ConfirmsBuffer, MAX_BUFF_AGE},
    deliveries::{Deliveries, PendingDeliveries},
    pending::Pending,
    received::{IdContinuity, Received},
};
use super::book::{Connection, ConnectionBook};
use crate::{record::DeliveryRecord, tasks::OutDatagram, Reliability, MAX_PACKAGE_SIZE};

mod confirms;
mod deliveries;
mod pending;
mod received;

#[derive(Clone)]
pub(crate) struct DeliveryHandler {
    book: Arc<Mutex<ConnectionBook<ConnDeliveryHandler>>>,
}

impl DeliveryHandler {
    pub(crate) fn new() -> Self {
        Self {
            book: Arc::new(Mutex::new(ConnectionBook::new())),
        }
    }

    pub(crate) async fn lock(&mut self) -> ReceiveHandlerGuard {
        ReceiveHandlerGuard {
            guard: self.book.lock().await,
        }
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
        let mut next = time + MAX_BUFF_AGE;
        let mut book = self.book.lock().await;

        while let Some((addr, handler)) = book.next() {
            let expiration = handler
                .confirms
                .send_confirms(time, force, addr, datagrams)
                .await?;
            next = next.min(expiration);
        }

        Ok(next)
    }

    pub(crate) async fn clean(&mut self, time: Instant) {
        self.book.lock().await.clean(time);
    }
}

/// The lock is unlocked once this guard is dropped.
pub(crate) struct ReceiveHandlerGuard<'a> {
    guard: MutexGuard<'a, ConnectionBook<ConnDeliveryHandler>>,
}

impl<'a> ReceiveHandlerGuard<'a> {
    /// Validate input package and return an iterator of to be delivered
    /// packages on success.
    ///
    /// All reliable sent packages are not to be delivered to the user
    /// directly but via the returned iterator.
    ///
    /// # Panics
    ///
    /// Panics if this is called with a non-reliable package.
    pub(crate) fn received<'buf>(
        &mut self,
        addr: SocketAddr,
        record: DeliveryRecord,
        data: Vec<u8>,
        buf: &'buf mut [u8],
    ) -> Result<Deliveries<'_, 'buf>, ReceivedIdError> {
        assert!(record.header().reliability().is_reliable());
        self.guard
            .update(record.time(), addr, ConnDeliveryHandler::new)
            .push(record, data, buf)
    }
}

struct ConnDeliveryHandler {
    received: Received,
    pending: Pending,
    confirms: ConfirmsBuffer,
}

impl ConnDeliveryHandler {
    fn new() -> Self {
        Self {
            received: Received::new(),
            pending: Pending::new(),
            confirms: ConfirmsBuffer::new(),
        }
    }

    /// Registers package as received and returns an iterator of the to be
    /// delivered packages.
    ///
    /// # Panics
    ///
    /// * If `buf` len is smaller than length of any of the drained buffered
    ///   pending package.
    ///
    /// * If `data` is longer than [`MAX_PACKAGE_SIZE`].
    fn push<'b>(
        &mut self,
        record: DeliveryRecord,
        data: Vec<u8>,
        buf: &'b mut [u8],
    ) -> Result<Deliveries<'_, 'b>, ReceivedIdError> {
        assert!(data.len() <= MAX_PACKAGE_SIZE);

        let result = self.received.process(record.header().id());
        if let Ok(_) | Err(ReceivedIdError::Duplicate) = result {
            // Push to the buffer even duplicate packages, because the reason
            // behind the re-delivery might be loss of the confirmation
            // datagram.
            self.confirms.push(record.time(), record.header().id());
        }

        Ok(match result? {
            IdContinuity::Continuous(bound) => Deliveries::drain(
                PendingDeliveries::new(bound, &mut self.pending),
                record,
                data,
                buf,
            ),
            IdContinuity::Sparse => match record.header().reliability() {
                Reliability::SemiOrdered => {
                    self.pending.store(record, &data);
                    Deliveries::empty(buf)
                }
                Reliability::Unordered => Deliveries::current(record, data, buf),
                Reliability::Unreliable => {
                    unreachable!("Unreliable packages cannot be processed by receive handler.")
                }
            },
        })
    }
}

impl Connection for ConnDeliveryHandler {
    fn pending(&self) -> bool {
        !self.confirms.is_empty()
    }
}
