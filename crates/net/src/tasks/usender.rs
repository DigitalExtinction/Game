use std::time::Instant;

use async_std::channel::{Receiver, Sender};
use tracing::{error, info};

use super::{cancellation::CancellationSender, dsender::OutDatagram};
use crate::{
    connection::Resends,
    header::{DatagramHeader, DatagramId},
    OutMessage,
};

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
    _cancellation: CancellationSender,
    datagrams: Sender<OutDatagram>,
    messages: Receiver<OutMessage>,
    mut resends: Resends,
) {
    info!("Starting user message sender on port {port}...");

    let mut counter = DatagramId::zero();

    loop {
        let Ok(message) = messages.recv().await else {
            break;
        };

        let header = DatagramHeader::new_data(message.reliable(), message.peers(), counter);
        counter = counter.incremented();

        if let DatagramHeader::Data(data_header) = header {
            if data_header.reliable() {
                let time = Instant::now();
                for target in &message.targets {
                    resends
                        .sent(
                            time,
                            target,
                            data_header.id(),
                            data_header.peers(),
                            &message.data,
                        )
                        .await;
                }
            }
        }

        let closed = datagrams
            .send(OutDatagram::new(header, message.data, message.targets))
            .await
            .is_err();

        if closed {
            error!("Datagram sender channel on port {port} is unexpectedly closed. ");
            break;
        }
    }

    info!("User message sender on port {port} finished.");
}
