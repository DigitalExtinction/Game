use std::time::{Duration, Instant};

use async_std::{channel::Receiver, future::timeout};
use de_messages::{BorrowedFromPlayers, FromGame, ToPlayers};
use de_net::{OutPackage, PackageSender, Peers};
use tracing::{error, info, warn};

use super::{message::InMessage, state::GameState};

pub(super) async fn run(
    port: u16,
    messages: Receiver<InMessage<ToPlayers>>,
    outputs: PackageSender,
    mut state: GameState,
) {
    info!("Starting game player package handler on port {port}...");

    'main: loop {
        if messages.is_closed() {
            break;
        }

        if outputs.is_closed() {
            error!("Outputs channel on port {port} was unexpectedly closed.");
            break;
        }

        let message = match timeout(Duration::from_millis(10), messages.recv()).await {
            Ok(Ok(message)) => Some(message),
            Ok(Err(_)) => break 'main,
            Err(_) => None,
        };

        if let Some(message) = message {
            let time = Instant::now();
            let meta = message.meta();
            let Some(player_id) = state.id(meta.source).await else {
                warn!(
                    "Received a player message from a non-participating client: {:?}.",
                    meta.source
                );

                let _ = outputs
                    .send(
                        OutPackage::encode_single(
                            &FromGame::NotJoined,
                            meta.reliability,
                            Peers::Server,
                            meta.source,
                        )
                        .unwrap(),
                    )
                    .await;
                continue;
            };

            let out_message = BorrowedFromPlayers::new(player_id, message.message());
            for buffer in state.lock().await.buffers_mut(Some(meta.source)) {
                if let Err(err) = buffer.push(meta.reliability, &out_message, time) {
                    warn!("Could not encode player message, skipping: {err:?}");
                }
            }
        }

        let mut guard = state.lock().await;
        let time = Instant::now();
        for buffer in guard.buffers_mut(None) {
            for output in buffer.build(time) {
                let result = outputs.send(output).await;
                if result.is_err() {
                    break 'main;
                }
            }
        }
    }

    info!("Game player package handler on port {port} finished.");
}
