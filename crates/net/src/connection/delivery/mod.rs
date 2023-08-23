use std::{net::SocketAddr, time::Instant};

use async_std::{
    channel::{SendError, Sender},
    sync::{Arc, Mutex},
};

use self::{
    confirms::{ConfirmsBuffer, MAX_BUFF_AGE},
    received::{Received, ReceivedIdError},
};
use super::book::{Connection, ConnectionBook};
use crate::{header::PackageId, tasks::OutDatagram};

mod confirms;
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
    ) -> Result<bool, ReceivedIdError> {
        self.book
            .lock()
            .await
            .update(time, addr, ConnDeliveryHandler::new)
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

struct ConnDeliveryHandler {
    received: Received,
    confirms: ConfirmsBuffer,
}

impl ConnDeliveryHandler {
    fn new() -> Self {
        Self {
            received: Received::new(),
            confirms: ConfirmsBuffer::new(),
        }
    }

    /// Registers a package as received and returns whether the it was a
    /// duplicate delivery.
    fn push(&mut self, time: Instant, id: PackageId) -> Result<bool, ReceivedIdError> {
        // Return early on error to avoid confirmation of erroneous datagrams.
        let duplicate = self.received.process(id)?;
        // Push to the buffer even duplicate packages, because the reason
        // behind the re-delivery might be loss of the confirmation datagram.
        self.confirms.push(time, id);
        Ok(duplicate)
    }
}

impl Connection for ConnDeliveryHandler {
    fn pending(&self) -> bool {
        !self.confirms.is_empty()
    }
}
