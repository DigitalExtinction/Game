use std::time::Duration;

use async_std::{channel::Sender, future::timeout};
use bincode::error::DecodeError;
use de_messages::{ToGame, ToPlayers};
use de_net::{InPackage, PackageReceiver, Peers};
use thiserror::Error;
use tracing::{error, info, warn};

use super::message::InMessage;

pub(super) async fn run(
    port: u16,
    packages: PackageReceiver,
    server: Sender<InMessage<ToGame>>,
    players: Sender<InMessage<ToPlayers>>,
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

        let peers = package.peers();
        let result = match peers {
            Peers::Server => handle_package(package, &server).await,
            Peers::Players => handle_package(package, &players).await,
        };

        if let Err(err) = result {
            match err {
                PackageHandleError::Decode(err) => {
                    warn!("Received invalid package: {err:?}");
                }
                PackageHandleError::SendError => {
                    if peers == Peers::Server {
                        break;
                    }
                }
            }
        }
    }

    info!("Game server input processor on port {port} finished.");
}

async fn handle_package<M>(
    package: InPackage,
    output: &Sender<InMessage<M>>,
) -> Result<(), PackageHandleError>
where
    M: bincode::Decode,
{
    for message_result in package.decode() {
        let message = message_result.map_err(PackageHandleError::from)?;
        output
            .send(InMessage::new(
                package.source(),
                package.reliability(),
                message,
            ))
            .await
            .map_err(|_| PackageHandleError::SendError)?;
    }

    Ok(())
}

#[derive(Debug, Error)]
enum PackageHandleError {
    #[error("Decoding error: {0}")]
    Decode(#[from] DecodeError),
    #[error("Sending to output channel failed.")]
    SendError,
}
