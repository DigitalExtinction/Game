use std::time::Instant;

use async_std::channel::{Receiver, Sender};
use tracing::info;

use super::dsender::OutDatagram;
use crate::{
    connection::Resends,
    header::{DatagramHeader, DatagramId},
    OutMessage,
};

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
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
                for &target in &message.targets {
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
            break;
        }
    }

    info!("User message sender on port {port} finished.");
}
