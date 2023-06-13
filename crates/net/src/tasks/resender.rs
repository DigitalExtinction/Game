use std::time::Instant;

use async_std::{channel::Sender, task};
use tracing::info;

use super::{communicator::ConnectionError, dsender::OutDatagram};
use crate::{connection::Resends, MAX_DATAGRAM_SIZE};

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
    mut datagrams: Sender<OutDatagram>,
    errors: Sender<ConnectionError>,
    mut resends: Resends,
) {
    info!("Starting resender on port {port}...");

    let mut buf = [0u8; MAX_DATAGRAM_SIZE];
    loop {
        resends.clean(Instant::now()).await;

        let Ok(resend_result) = resends
            .resend(Instant::now(), &mut buf, &mut datagrams)
            .await
        else {
            break;
        };

        for target in resend_result.failures {
            let result = errors.send(ConnectionError::new(target)).await;
            if result.is_err() {
                break;
            }
        }

        let now = Instant::now();
        if resend_result.next > now {
            task::sleep(resend_result.next - now).await;
        }
    }

    info!("Resender on port {port} finished.");
}
