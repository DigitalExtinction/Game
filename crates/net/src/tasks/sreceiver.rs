use std::time::{Duration, Instant};

use async_std::{channel::Receiver, future::timeout};
use tracing::{error, info};

use super::{cancellation::CancellationRecv, dreceiver::InSystemDatagram};
use crate::connection::DispatchHandler;

/// Handler of protocol control datagrams.
///
/// The handler runs a loop which finishes when `datagrams` channel is closed.
pub(super) async fn run(
    port: u16,
    cancellation: CancellationRecv,
    datagrams: Receiver<InSystemDatagram>,
    mut dispatch_handler: DispatchHandler,
) {
    info!("Starting protocol control datagram receiver on port {port}...");

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

        dispatch_handler
            .confirmed(Instant::now(), datagram.source, &datagram.data)
            .await;
    }

    info!("Protocol control datagram receiver on port {port} finished.");
}
