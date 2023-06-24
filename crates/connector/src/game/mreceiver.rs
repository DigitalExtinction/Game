use std::time::Duration;

use async_std::{channel::Sender, future::timeout};
use de_net::{MessageReceiver, Peers};
use tracing::{error, info, warn};

use super::greceiver::ToGameMessage;
use crate::game::preceiver::PlayersMessage;

pub(super) async fn run(
    port: u16,
    messages: MessageReceiver,
    server: Sender<ToGameMessage>,
    players: Sender<PlayersMessage>,
) {
    info!("Starting game server input processor on port {port}...");

    loop {
        if server.is_closed() {
            break;
        }

        if players.is_closed() {
            error!("Players channel on port {port} was unexpectedly closed.");
            break;
        }

        let Ok(message) = timeout(Duration::from_millis(500), messages.recv()).await else {
            continue;
        };

        let Ok(message) = message else {
            error!("Inputs channel on port {port} was unexpectedly closed.");
            break;
        };

        match message.peers() {
            Peers::Server => {
                for item in message.decode() {
                    match item {
                        Ok(item) => {
                            let result = server
                                .send(ToGameMessage::new(
                                    message.source(),
                                    message.reliable(),
                                    item,
                                ))
                                .await;
                            if result.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            warn!("Received invalid message: {err:?}");
                            break;
                        }
                    }
                }
            }
            Peers::Players => {
                let _ = players
                    .send(PlayersMessage::new(
                        message.reliable(),
                        message.source(),
                        message.data(),
                    ))
                    .await;
            }
        }
    }

    info!("Game server input processor on port {port} finished.");
}
