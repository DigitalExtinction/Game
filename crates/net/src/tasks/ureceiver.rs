use std::time::{Duration, Instant};

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
};
use tracing::{error, info, trace, warn};

use super::{cancellation::CancellationSender, dreceiver::InPackageDatagram};
use crate::{connection::Confirmations, InPackage};

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
    mut confirms: Confirmations,
) {
    info!("Starting package receiver on port {port}...");

    loop {
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

        if datagram.header.reliable() {
            match confirms
                .received(Instant::now(), datagram.source, datagram.header.id())
                .await
            {
                Ok(true) => {
                    trace!(
                        "Duplicate delivery of package {:?} from {:?}.",
                        datagram.header.id(),
                        datagram.source
                    );
                    continue;
                }
                Ok(false) => (),
                Err(err) => {
                    warn!("Package ID error: {err:?}");
                }
            }
        }

        let result = packages
            .send(InPackage::new(
                datagram.data,
                datagram.header.reliable(),
                datagram.header.peers(),
                datagram.source,
            ))
            .await;

        if result.is_err() {
            break;
        }
    }

    info!("Package receiver on port {port} finished.");
}
