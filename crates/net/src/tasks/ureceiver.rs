use std::time::{Duration, Instant};

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
};
use tracing::{error, info};

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
                break;
            } else {
                continue;
            }
        };

        let Ok(datagram) = result else {
            error!("Datagram receiver channel is unexpectedly closed.");
            break;
        };

        if datagram.header.reliable() {
            confirms
                .received(Instant::now(), datagram.source, datagram.header.id())
                .await;
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
