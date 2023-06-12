//! System message receiver

use std::time::Instant;

use async_std::channel::Receiver;
use tracing::info;

use super::dreceiver::InSystemDatagram;
use crate::connection::Resends;

/// Handler of system (protocol) datagrams.
///
/// The handler runs a loop which finishes when `datagrams` channel is closed.
pub(crate) async fn run(port: u16, datagrams: Receiver<InSystemDatagram>, mut resends: Resends) {
    info!("Starting system message receiver on port {port}...");

    loop {
        let Ok(datagram) = datagrams.recv().await else {
            break;
        };

        resends
            .confirmed(Instant::now(), datagram.source, &datagram.data)
            .await;
    }

    info!("System message receiver on port {port} finished.");
}
