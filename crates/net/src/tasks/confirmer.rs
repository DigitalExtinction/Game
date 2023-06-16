use std::time::Instant;

use async_std::{channel::Sender, task};
use tracing::info;

use super::{cancellation::CancellationRecv, dsender::OutDatagram};
use crate::connection::Confirmations;

/// Scheduler of datagram confirmations.
pub(super) async fn run(
    port: u16,
    cancellation: CancellationRecv,
    mut datagrams: Sender<OutDatagram>,
    mut confirms: Confirmations,
) {
    info!("Starting confirmer on port {port}...");

    loop {
        if datagrams.is_closed() {
            break;
        }

        confirms.clean(Instant::now()).await;

        let Ok(next) = confirms
            .send_confirms(Instant::now(), cancellation.cancelled(), &mut datagrams)
            .await
        else {
            break;
        };

        if cancellation.cancelled() {
            break;
        }

        let now = Instant::now();
        if next > now {
            task::sleep(next - now).await;
        }
    }

    info!("Confirmer on port {port} finished.");
}
