use std::time::Duration;

use async_std::{channel::Sender, future::timeout};
use de_messages::ToGame;
use de_net::{ConnErrorReceiver, Reliability};
use tracing::{error, info, warn};

use super::greceiver::ToGameMessage;

pub(super) async fn run(port: u16, errors: ConnErrorReceiver, server: Sender<ToGameMessage>) {
    info!("Starting game connection error handler on port {port}...");

    loop {
        if server.is_closed() {
            break;
        }

        let Ok(error) = timeout(Duration::from_millis(500), errors.recv()).await else {
            continue;
        };

        let Ok(error) = error else {
            error!("Errors channel on port {port} was unexpectedly closed.");
            break;
        };

        warn!("In game connection lost with {:?}", error.target());
        let _ = server
            .send(ToGameMessage::new(
                error.target(),
                Reliability::Unordered,
                ToGame::Leave,
            ))
            .await;
    }

    info!("Game connection error handler on port {port} finished.");
}
