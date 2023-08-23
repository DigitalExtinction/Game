use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use async_std::{
    channel::{SendError, Sender},
    sync::{Arc, Mutex},
};

use self::resends::{RescheduleResult, Resends, START_BACKOFF_MS};
use super::book::{Connection, ConnectionBook};
use crate::{
    header::{DatagramHeader, PackageHeader, PackageId, PackageIdRange},
    tasks::OutDatagram,
};

mod resends;

#[derive(Clone)]
pub(crate) struct DispatchHandler {
    book: Arc<Mutex<ConnectionBook<ConnDispatchHandler>>>,
}

impl DispatchHandler {
    pub(crate) fn new() -> Self {
        Self {
            book: Arc::new(Mutex::new(ConnectionBook::new())),
        }
    }

    /// Returns ID to be given to a to-be-send package.
    ///
    /// It is assumed that this is called exactly once before each reliably
    /// send package and that all generated IDs are used.
    pub(crate) async fn next_package_id(&mut self, time: Instant, addr: SocketAddr) -> PackageId {
        let mut book = self.book.lock().await;
        let handler = book.update(time, addr, ConnDispatchHandler::new);
        handler.next_package_id()
    }

    pub(crate) async fn sent(
        &mut self,
        time: Instant,
        addr: SocketAddr,
        header: PackageHeader,
        data: &[u8],
    ) {
        let mut book = self.book.lock().await;
        let handler = book.update(time, addr, ConnDispatchHandler::new);
        handler.resends.push(header, data, time);
    }

    /// Processes data with package confirmations.
    ///
    /// The data encode IDs of delivered (and confirmed) packages so that they
    /// can be forgotten.
    pub(crate) async fn confirmed(&mut self, time: Instant, addr: SocketAddr, data: &[u8]) {
        let mut book = self.book.lock().await;
        let handler = book.update(time, addr, ConnDispatchHandler::new);

        for i in 0..data.len() / 3 {
            let offset = i * 3;
            let id = PackageId::from_bytes(&data[offset..offset + 3]);
            handler.resends.resolve(id);
        }
    }

    /// Re-send all packages already due for re-sending.
    pub(crate) async fn resend(
        &mut self,
        time: Instant,
        buf: &mut [u8],
        datagrams: &mut Sender<OutDatagram>,
    ) -> Result<ResendResult, SendError<OutDatagram>> {
        let mut result = ResendResult {
            failures: Vec::new(),
            pending: 0,
            next: time + Duration::from_millis(START_BACKOFF_MS),
        };

        let mut book = self.book.lock().await;

        while let Some((addr, handler)) = book.next() {
            let failure = loop {
                match handler.resends.reschedule(buf, time) {
                    RescheduleResult::Resend { len, header } => {
                        datagrams
                            .send(OutDatagram::new(
                                DatagramHeader::Package(header),
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
            } else {
                result.pending += handler.resends.len();
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
    /// Number of pending (not yet confirmed) datagrams.
    pub(crate) pending: usize,
    /// Soonest possible time of the next datagram resend.
    pub(crate) next: Instant,
}

struct ConnDispatchHandler {
    resends: Resends,
    package_ids: PackageIdRange,
}

impl ConnDispatchHandler {
    fn new() -> Self {
        Self {
            resends: Resends::new(),
            package_ids: PackageIdRange::counter(),
        }
    }

    fn next_package_id(&mut self) -> PackageId {
        self.package_ids.next().unwrap()
    }
}

impl Connection for ConnDispatchHandler {
    fn pending(&self) -> bool {
        !self.resends.is_empty()
    }
}
