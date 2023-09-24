use std::time::Duration;

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
};
use tracing::{error, info, trace, warn};

use super::{cancellation::CancellationSender, dreceiver::InPackageDatagram};
use crate::{
    connection::{DeliveryHandler, ReceivedIdError},
    record::DeliveryRecord,
    InPackage, MAX_PACKAGE_SIZE,
};

/// Handler of user datagrams, i.e. datagrams with user data targeted to
/// higher-level users of the network protocol.
///
/// The handler runs a loop which finishes when `datagrams` or `packages`
/// channel is closed.
pub(super) async fn run(
    port: u16,
    _cancellation: CancellationSender,
    datagrams: Receiver<InPackageDatagram>,
    packages: Sender<InPackage>,
    mut delivery_handler: DeliveryHandler,
) {
    info!("Starting package receiver on port {port}...");

    let mut buf = vec![0; MAX_PACKAGE_SIZE];

    'main: loop {
        let Ok(result) = timeout(Duration::from_millis(500), datagrams.recv()).await else {
            if packages.is_closed() {
                // This must be here in case of no incoming packages to ensure
                // that the check is done at least once every 500ms.
                break;
            } else {
                continue;
            }
        };

        // This must be here in case of both a) packages are incoming
        // frequently (so no timeouts above), b) packages are skipped because
        // they are duplicates (so no packages.send() is called).
        if packages.is_closed() {
            break;
        }

        let Ok(datagram) = result else {
            error!("Datagram receiver channel is unexpectedly closed.");
            break;
        };
        let record = DeliveryRecord::now(datagram.header);

        if datagram.header.reliability().is_reliable() {
            let mut guard = delivery_handler.lock().await;
            match guard.received(datagram.source, record, datagram.data, &mut buf) {
                Ok(deliveries) => {
                    for (record, data) in deliveries {
                        let result = packages
                            .send(InPackage::new(
                                data,
                                record.header().reliability(),
                                record.header().peers(),
                                datagram.source,
                                record.time(),
                            ))
                            .await;
                        if result.is_err() {
                            break 'main;
                        }
                    }
                }
                Err(ReceivedIdError::Duplicate) => {
                    trace!(
                        "Duplicate delivery of package {:?} from {:?}.",
                        datagram.header.id(),
                        datagram.source
                    );
                }
                Err(err) => {
                    warn!("Received package error: {err:?}");
                }
            }
        } else {
            let result = packages
                .send(InPackage::new(
                    datagram.data,
                    datagram.header.reliability(),
                    datagram.header.peers(),
                    datagram.source,
                    record.time(),
                ))
                .await;
            if result.is_err() {
                break 'main;
            }
        }
    }

    info!("Package receiver on port {port} finished.");
}
