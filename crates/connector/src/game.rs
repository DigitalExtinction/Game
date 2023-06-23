use std::{net::SocketAddr, time::Duration};

use ahash::AHashSet;
use anyhow::Context;
use async_std::{channel::TryRecvError, prelude::FutureExt as StdFutureExt};
use de_net::{
    self, ConnErrorReceiver, InMessage, MessageReceiver, MessageSender, Network, OutMessage, Peers,
};
use tracing::info;

pub(crate) struct GameProcessor {
    outs: MessageSender,
    ins: MessageReceiver,
    conn_errs: ConnErrorReceiver,
    players: AHashSet<SocketAddr>,
}

impl GameProcessor {
    pub(crate) async fn start(port: u16) -> anyhow::Result<()> {
        let net = Network::bind(Some(port))
            .await
            .with_context(|| format!("Failed to bind on port {port}"))?;
        info!("Listening on port {}", port);

        let (outs, ins, conn_errs) = de_net::startup(net);
        let processor = Self {
            outs,
            ins,
            conn_errs,
            players: AHashSet::new(),
        };

        processor.run().await
    }

    async fn run(mut self) -> anyhow::Result<()> {
        loop {
            if let Ok(input_result) = self.ins.recv().timeout(Duration::from_millis(1000)).await {
                let message = input_result.context("Data receiving failed")?;
                self.players.insert(message.source());

                match message.peers() {
                    Peers::Players => {
                        self.handle_players(message).await?;
                    }
                    Peers::Server => todo!("Not yet implemented"),
                }
            }

            let error = self.conn_errs.try_recv();
            if matches!(error, Err(TryRecvError::Empty)) {
                continue;
            }

            let error = error.context("Errors receiving failed")?;
            self.players.remove(&error.target());
        }
    }

    async fn handle_players(&mut self, message: InMessage) -> anyhow::Result<()> {
        let reliable = message.reliable();

        let targets: Vec<SocketAddr> = self
            .players
            .iter()
            .cloned()
            .filter(|&target| target != message.source())
            .collect();

        self.outs
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
