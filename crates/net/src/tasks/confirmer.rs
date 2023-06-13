use std::time::Instant;

use async_std::{channel::Sender, task};
use tracing::info;

use super::dsender::OutDatagram;
use crate::connection::Confirmations;

/// Scheduler of datagram confirmations.
pub(super) async fn run(
    port: u16,
    mut datagrams: Sender<OutDatagram>,
    mut confirms: Confirmations,
) {
    info!("Starting confirmer on port {port}...");

    loop {
        confirms.clean(Instant::now()).await;

        let Ok(next) = confirms
            .send_confirms(Instant::now(), &mut datagrams)
            .await
        else {
            break;
        };

        let now = Instant::now();
        if next > now {
            task::sleep(next - now).await;
        }
    }

    info!("Confirmer on port {port} finished.");
}
