use std::{net::SocketAddr, time::Duration};

use ahash::AHashSet;
use anyhow::Context;
use async_std::{channel::TryRecvError, prelude::FutureExt as StdFutureExt};
use de_net::{self, Communicator, InMessage, Network, OutMessage, Peers};
use tracing::info;

pub(crate) struct GameProcessor {
    communicator: Communicator,
    players: AHashSet<SocketAddr>,
}

impl GameProcessor {
    pub(crate) async fn start(port: u16) -> anyhow::Result<()> {
        let net = Network::bind(Some(port))
            .await
            .with_context(|| format!("Failed to bind on port {port}"))?;
        info!("Listening on port {}", port);

        let processor = Self {
            communicator: de_net::startup(net),
            players: AHashSet::new(),
        };

        processor.run().await
    }

    async fn run(mut self) -> anyhow::Result<()> {
        loop {
            if let Ok(input_result) = self
                .communicator
                .recv()
                .timeout(Duration::from_millis(1000))
                .await
            {
                let message = input_result.context("Data receiving failed")?;
                self.players.insert(message.source());

                match message.peers() {
                    Peers::Players => {
                        self.handle_players(message).await?;
                    }
                    Peers::Server => todo!("Not yet implemented"),
                }
            }

            let error = self.communicator.errors();
            if matches!(error, Err(TryRecvError::Empty)) {
                continue;
            }

            let error = error.context("Errors receiving failed")?;
            self.players.remove(&error.target());
        }
    }

    async fn handle_players(&mut self, message: InMessage) -> anyhow::Result<()> {
        let reliable = message.reliable();

        let targets = self
            .players
            .iter()
            .cloned()
            .filter(|&target| target != message.source())
            .collect();

        self.communicator
            .send(OutMessage::new(
                message.data(),
                reliable,
                Peers::Players,
                targets,
            ))
            .await
            .context("Data sending failed")
    }
}
