use std::time::Duration;

use async_std::{channel::Sender, future::timeout};
use de_net::{PackageReceiver, Peers};
use tracing::{error, info, warn};

use super::greceiver::ToGameMessage;
use crate::game::preceiver::PlayersPackage;

pub(super) async fn run(
    port: u16,
    packages: PackageReceiver,
    server: Sender<ToGameMessage>,
    players: Sender<PlayersPackage>,
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

        let Ok(package) = timeout(Duration::from_millis(500), packages.recv()).await else {
            continue;
        };

        let Ok(package) = package else {
            error!("Inputs channel on port {port} was unexpectedly closed.");
            break;
        };

        match package.peers() {
            Peers::Server => {
                for message_result in package.decode() {
                    match message_result {
                        Ok(message) => {
                            let result = server
                                .send(ToGameMessage::new(
                                    package.source(),
                                    package.reliable(),
                                    message,
                                ))
                                .await;
                            if result.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            warn!("Received invalid package: {err:?}");
                            break;
                        }
                    }
                }
            }
            Peers::Players => {
                let _ = players
                    .send(PlayersPackage::new(
                        package.reliable(),
                        package.source(),
                        package.data(),
                    ))
                    .await;
            }
        }
    }

    info!("Game server input processor on port {port} finished.");
}
