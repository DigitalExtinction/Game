use std::{net::SocketAddr, time::Duration};

use async_std::{
    channel::{Receiver, Sender},
    task,
};
use de_messages::{FromGame, JoinError, Readiness, ToGame};
use de_net::{OutPackage, Peers, Reliability};
use tracing::{error, info, warn};

use super::{
    message::{InMessage, MessageMeta},
    state::{GameState, JoinError as JoinErrorInner},
};
use crate::clients::Clients;

pub(super) struct GameProcessor {
    port: u16,
    owner: SocketAddr,
    messages: Receiver<InMessage<ToGame>>,
    outputs: Sender<OutPackage>,
    state: GameState,
    clients: Clients,
}

impl GameProcessor {
    pub(super) fn new(
        port: u16,
        owner: SocketAddr,
        messages: Receiver<InMessage<ToGame>>,
        outputs: Sender<OutPackage>,
        state: GameState,
        clients: Clients,
    ) -> Self {
        Self {
            port,
            owner,
            messages,
            outputs,
            state,
            clients,
        }
    }

    pub(super) async fn run(mut self) {
        info!(
            "Starting game server message handler on port {}...",
            self.port
        );

        // Wait a little to ensure that game creation message (send from main
        // server) is delivered first.
        task::sleep(Duration::from_millis(100)).await;
        self.join(self.owner).await.unwrap();

        loop {
            if self.outputs.is_closed() {
                error!(
                    "Output message channel on port {} is unexpectedly closed.",
                    self.port
                );
                break;
            }

            let Ok(message) = self.messages.recv().await else {
                error!(
                    "Game message channel on port {} is unexpectedly closed.",
                    self.port
                );
                break;
            };

            if self.handle_ignore(&message).await {
                continue;
            }

            match message.message() {
                ToGame::Ping(id) => {
                    self.process_ping(message.meta(), *id).await;
                }
                ToGame::Join => {
                    self.process_join(message.meta()).await;
                }
                ToGame::Leave => {
                    self.process_leave(message.meta()).await;
                }
                ToGame::Readiness(readiness) => {
                    self.process_readiness(message.meta(), *readiness).await;
                }
            }

            if self.state.is_empty().await {
                info!("Everybody disconnected, quitting...");
                break;
            }
        }

        info!(
            "Game server message handler on port {} finished.",
            self.port
        );
    }

    /// Returns true if the massage should be ignored and further handles such
    /// messages.
    async fn handle_ignore(&self, message: &InMessage<ToGame>) -> bool {
        if matches!(message.message(), ToGame::Join | ToGame::Leave) {
            // Join must be excluded from the condition because of the
            // chicken and egg problem.
            //
            // Leave must be excluded due to possibility that the message
            // was redelivered.
            return false;
        }

        if self.state.contains(message.meta().source).await {
            return false;
        }

        warn!(
            "Received a game message from a non-participating client: {:?}.",
            message.meta().source
        );
        let _ = self
            .outputs
            .send(
                OutPackage::encode_single(
                    &FromGame::NotJoined,
                    message.meta().reliability,
                    Peers::Server,
                    message.meta().source,
                )
                .unwrap(),
            )
            .await;
        true
    }

    /// Process a ping message.
    async fn process_ping(&self, meta: MessageMeta, id: u32) {
        let _ = self
            .outputs
            .send(
                OutPackage::encode_single(
                    &FromGame::Pong(id),
                    meta.reliability,
                    Peers::Server,
                    meta.source,
                )
                .unwrap(),
            )
            .await;
    }

    /// Process connect message.
    async fn process_join(&mut self, meta: MessageMeta) {
        if let Err(err) = self.clients.reserve(meta.source).await {
            warn!("Join request error: {err}");
            self.send(
                &FromGame::JoinError(JoinError::DifferentGame),
                Reliability::Unordered,
                meta.source,
            )
            .await;
            return;
        }

        match self.join(meta.source).await {
            Ok(_) => {
                self.clients.set(meta.source, self.port).await;
            }
            Err(err) => {
                self.clients.free(meta.source).await;

                match err {
                    JoinErrorInner::AlreadyJoined => {
                        warn!(
                            "Player {:?} has already joined game on port {}.",
                            meta.source, self.port
                        );

                        self.send(
                            &FromGame::JoinError(JoinError::AlreadyJoined),
                            Reliability::Unordered,
                            meta.source,
                        )
                        .await;
                    }
                    JoinErrorInner::GameFull => {
                        warn!(
                            "Player {:?} could not join game on port {} because the game is full.",
                            meta.source, self.port
                        );

                        self.send(
                            &FromGame::JoinError(JoinError::GameFull),
                            Reliability::Unordered,
                            meta.source,
                        )
                        .await;
                    }
                    JoinErrorInner::GameNotOpened => {
                        warn!(
                            "Player {:?} could not join game on port {} because the game is no \
                             longer opened.",
                            meta.source, self.port
                        );

                        self.send(
                            &FromGame::JoinError(JoinError::GameNotOpened),
                            Reliability::Unordered,
                            meta.source,
                        )
                        .await;
                    }
                }
            }
        }
    }

    async fn join(&mut self, addr: SocketAddr) -> Result<(), JoinErrorInner> {
        let id = self.state.add(addr).await?;
        info!(
            "Player {id} on {addr:?} just joined game on port {}.",
            self.port
        );
        self.send(&FromGame::Joined(id), Reliability::SemiOrdered, addr)
            .await;
        self.send_all(
            &FromGame::PeerJoined(id),
            Reliability::SemiOrdered,
            Some(addr),
        )
        .await;
        Ok(())
    }

    /// Process disconnect message.
    async fn process_leave(&mut self, meta: MessageMeta) {
        let Some(mut player_state) = self.state.remove(meta.source).await else {
            warn!("Tried to remove non-existent player {:?}.", meta.source);
            return;
        };

        self.clients.free(meta.source).await;

        info!(
            "Player {} on {:?} just left game on port {}.",
            player_state.id(),
            meta.source,
            self.port
        );

        for output in player_state.buffer_mut().build_all() {
            let _ = self.outputs.send(output).await;
        }

        self.send(&FromGame::Left, Reliability::SemiOrdered, meta.source)
            .await;
        self.send_all(
            &FromGame::PeerLeft(player_state.id()),
            Reliability::SemiOrdered,
            None,
        )
        .await;
    }

    async fn process_readiness(&mut self, meta: MessageMeta, readiness: Readiness) {
        match self.state.update_readiness(meta.source, readiness).await {
            Ok(progressed) => {
                if progressed {
                    self.send_all(
                        &FromGame::GameReadiness(readiness),
                        Reliability::SemiOrdered,
                        None,
                    )
                    .await;
                }
            }
            Err(err) => warn!(
                "Invalid readiness update from {source:?}: {err:?}",
                source = meta.source
            ),
        }
    }

    /// Send a reliable message to all players of the game.
    ///
    /// # Arguments
    ///
    /// * `message` - message to be sent.
    ///
    /// * `reliability` - reliability mode for the message.
    ///
    /// * `exclude` - if not None, the message will be delivered to all but
    ///   this player.
    async fn send_all<E>(&self, message: &E, reliability: Reliability, exclude: Option<SocketAddr>)
    where
        E: bincode::Encode,
    {
        for target in self.state.targets(exclude).await {
            self.send(message, reliability, target).await;
        }
    }

    async fn send<E>(&self, message: &E, reliability: Reliability, target: SocketAddr)
    where
        E: bincode::Encode,
    {
        let message =
            OutPackage::encode_single(message, reliability, Peers::Server, target).unwrap();
        let _ = self.outputs.send(message).await;
    }
}
