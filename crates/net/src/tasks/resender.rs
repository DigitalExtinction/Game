use std::time::{Duration, Instant};

use async_std::{channel::Sender, task};
use tracing::{error, info};

use super::{
    cancellation::{CancellationRecv, CancellationSender},
    communicator::ConnectionError,
    dsender::OutDatagram,
};
use crate::{connection::DispatchHandler, MAX_DATAGRAM_SIZE};

const CANCELLATION_DEADLINE: Duration = Duration::from_secs(5);

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
    cancellation_recv: CancellationRecv,
    _cancellation_send: CancellationSender,
    mut datagrams: Sender<OutDatagram>,
    errors: Sender<ConnectionError>,
    mut dispatch_handler: DispatchHandler,
) {
    info!("Starting resender on port {port}...");

    let mut buf = [0u8; MAX_DATAGRAM_SIZE];
    let mut deadline = None;

    'main: loop {
        if deadline.is_none() && cancellation_recv.cancelled() && errors.is_closed() {
            deadline = Some(Instant::now() + CANCELLATION_DEADLINE);
        }

        dispatch_handler.clean(Instant::now()).await;

        let Ok(resend_result) = dispatch_handler
            .resend(Instant::now(), &mut buf, &mut datagrams)
            .await
        else {
            error!("Datagram sender channel on port {port} is unexpectedly closed.");
            break;
        };

        if !errors.is_closed() {
            'failures: for target in resend_result.failures {
                let result = errors.send(ConnectionError::new(target)).await;
                if result.is_err() {
                    if cancellation_recv.cancelled() {
                        break 'main;
                    } else {
                        break 'failures;
                    }
                }
            }
        }

        if let Some(deadline) = deadline {
            if deadline < resend_result.next || resend_result.pending == 0 {
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
