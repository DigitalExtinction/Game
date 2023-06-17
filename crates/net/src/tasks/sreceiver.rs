//! System message receiver

use std::time::{Duration, Instant};

use async_std::{channel::Receiver, future::timeout};
use tracing::{error, info};

use super::{cancellation::CancellationRecv, dreceiver::InSystemDatagram};
use crate::connection::Resends;

/// Handler of system (protocol) datagrams.
///
/// The handler runs a loop which finishes when `datagrams` channel is closed.
pub(super) async fn run(
    port: u16,
    cancellation: CancellationRecv,
    datagrams: Receiver<InSystemDatagram>,
    mut resends: Resends,
) {
    info!("Starting system message receiver on port {port}...");

    loop {
        if cancellation.cancelled() {
            break;
        }

        let Ok(result) = timeout(Duration::from_millis(500), datagrams.recv()).await else {
            continue;
        };

        let Ok(datagram) = result else {
            error!("Datagram receiver channel on port {port} is unexpectedly closed.");
            break;
        };

        resends
            .confirmed(Instant::now(), datagram.source, &datagram.data)
            .await;
    }

    info!("System message receiver on port {port} finished.");
}
