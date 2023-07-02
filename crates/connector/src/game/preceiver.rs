use std::net::SocketAddr;

use async_std::channel::Receiver;
use de_net::{PackageSender, OutPackage, Peers};
use tracing::{error, info};

use super::state::GameState;

/// A data message destined to other players in the game.
pub(super) struct PlayersMessage {
    reliable: bool,
    source: SocketAddr,
    data: Vec<u8>,
}

impl PlayersMessage {
    pub(super) fn new(reliable: bool, source: SocketAddr, data: Vec<u8>) -> Self {
        Self {
            reliable,
            source,
            data,
        }
    }
}

pub(super) async fn run(
    port: u16,
    messages: Receiver<PlayersMessage>,
    outputs: PackageSender,
    state: GameState,
) {
    info!("Starting game player message handler on port {port}...");

    loop {
        if messages.is_closed() {
            break;
        }

        if outputs.is_closed() {
            error!("Outputs channel on port {port} was unexpectedly closed.");
            break;
        }

        let Ok(message) = messages.recv().await else {
            break;
        };

        let Some(targets) = state.targets(Some(message.source)).await else {
            continue;
        };

        let result = outputs
            .send(OutPackage::new(
                message.data,
                message.reliable,
                Peers::Players,
                targets,
            ))
            .await;
        if result.is_err() {
            break;
        }
    }

    info!("Game player message handler on port {port} finished.");
}
