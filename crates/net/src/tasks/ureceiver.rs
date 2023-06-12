//! User message receiver
use std::time::{Duration, Instant};

use async_std::{
    channel::{Receiver, Sender},
    future::timeout,
};
use tracing::{error, info};

use super::dreceiver::InUserDatagram;
use crate::{connection::Confirmations, InMessage};

/// Handler of user datagrams, i.e. datagrams with user data targeted to
/// higher-level users of the network protocol.
///
/// The handler runs a loop which finishes when `datagrams` or `messages`
/// channel is closed.
pub(crate) async fn run(
    port: u16,
    datagrams: Receiver<InUserDatagram>,
    messages: Sender<InMessage>,
    mut confirms: Confirmations,
) {
    info!("Starting user message receiver on port {port}...");

    loop {
        let Ok(result) = timeout(Duration::from_millis(500), datagrams.recv()).await else {
            if messages.is_closed() {
                break;
            } else {
                continue;
            }
        };

        let Ok(datagram) = result else {
            error!("Datagram input channel is unexpectedly closed.");
            break;
        };

        if datagram.header.reliable() {
            confirms
                .received(Instant::now(), datagram.source, datagram.header.id())
                .await;
        }

        let result = messages
            .send(InMessage::new(
                datagram.data,
                datagram.header.reliable(),
                datagram.header.peers(),
                datagram.source,
            ))
            .await;

        if result.is_err() {
            break;
        }
    }

    info!("User message receiver on port {port} finished.");
}
