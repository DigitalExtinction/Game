use std::net::SocketAddr;

use anyhow::Context;
use async_std::task;
use de_net::{
    self, FromServer, MessageDecoder, PackageReceiver, PackageSender, OutPackage, Peers, Socket,
    ToServer,
};
use tracing::{error, info, warn};

use crate::game;

/// Main game server responsible for initial communication with clients and
/// establishment of game sub-servers.
pub(crate) struct MainServer {
    outputs: PackageSender,
    inputs: PackageReceiver,
}

impl MainServer {
    /// Setup the server & startup its network stack.
    pub(crate) fn start(net: Socket) -> Self {
        let (outputs, inputs, _) = de_net::startup(
            |t| {
                task::spawn(t);
            },
            net,
        );
        Self { outputs, inputs }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let message = self
                .inputs
                .recv()
                .await
                .context("Inputs channel unexpectedly closed")?;

            match message.peers() {
                Peers::Players => {
                    warn!("Message for players unexpectedly received.");
                }
                Peers::Server => {
                    self.process(message.source(), message.decode()).await?;
                }
            }
        }
    }

    async fn process(
        &mut self,
        source: SocketAddr,
        messages: MessageDecoder<'_, ToServer>,
    ) -> anyhow::Result<()> {
        for message in messages {
            let Ok(message) = message else {
                warn!("Invalid message received");
                return Ok(());
            };

            match message {
                ToServer::Ping(id) => self.reply(&FromServer::Pong(id), source).await?,
                ToServer::OpenGame => self.open_game(source).await?,
            }
        }

        Ok(())
    }

    async fn open_game(&mut self, source: SocketAddr) -> anyhow::Result<()> {
        match Socket::bind(None).await {
            Ok(net) => {
                let port = net.port();
                info!("Starting new game on port {port}.");
                self.reply(&FromServer::GameOpened { port }, source).await?;
                game::startup(net, source).await;
                Ok(())
            }
            Err(error) => {
                error!("Failed to open a new game: {:?}", error);
                Ok(())
            }
        }
    }

    async fn reply(&mut self, message: &FromServer, target: SocketAddr) -> anyhow::Result<()> {
        self.outputs
            .send(OutPackage::encode_single(message, true, Peers::Server, target).unwrap())
            .await
            .context("Failed to send a reply")
    }
}
