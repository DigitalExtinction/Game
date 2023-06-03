use std::{net::SocketAddr, time::Duration};

use ahash::AHashSet;
use anyhow::Context;
use async_std::{channel::TryRecvError, prelude::FutureExt as StdFutureExt, task};
use de_net::{setup_processor, Communicator, InMessage, Network, OutMessage, Peers};
use tracing::info;

const PORT: u16 = 8082;

pub fn start() {
    info!("Starting...");

    task::block_on(task::spawn(async {
        if let Err(error) = GameProcessor::start().await {
            eprintln!("{:?}", error);
        }
    }));
}

struct GameProcessor {
    communicator: Communicator,
    players: AHashSet<SocketAddr>,
}

impl GameProcessor {
    async fn start() -> anyhow::Result<()> {
        let net = Network::bind(Some(PORT))
            .await
            .with_context(|| format!("Failed to bind on port {PORT}"))?;
        info!("Listening on port {}", PORT);

        let (communicator, processor) = setup_processor(net);
        task::spawn(processor.run());

        let processor = Self {
            communicator,
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
