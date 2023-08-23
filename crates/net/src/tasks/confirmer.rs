use std::time::Instant;

use async_std::{channel::Sender, task};
use tracing::{error, info};

use super::{cancellation::CancellationRecv, dsender::OutDatagram};
use crate::connection::DeliveryHandler;

/// Scheduler of datagram confirmations.
pub(super) async fn run(
    port: u16,
    cancellation: CancellationRecv,
    mut datagrams: Sender<OutDatagram>,
    mut delivery_handler: DeliveryHandler,
) {
    info!("Starting confirmer on port {port}...");

    loop {
        delivery_handler.clean(Instant::now()).await;

        let Ok(next) = delivery_handler
            .send_confirms(Instant::now(), cancellation.cancelled(), &mut datagrams)
            .await
        else {
            error!("Datagram sender channel on port {port} is unexpectedly closed.");
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
