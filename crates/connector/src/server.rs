use std::net::SocketAddr;

use anyhow::Context;
use async_std::task;
use de_net::{
    self, FromServer, MessageDecoder, OutPackage, PackageReceiver, PackageSender, Peers, Socket,
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
    pub(crate) fn start(socket: Socket) -> Self {
        let (outputs, inputs, _) = de_net::startup(
            |t| {
                task::spawn(t);
            },
            socket,
        );
        Self { outputs, inputs }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let package = self
                .inputs
                .recv()
                .await
                .context("Inputs channel unexpectedly closed")?;

            match package.peers() {
                Peers::Players => {
                    warn!("Package for players unexpectedly received.");
                }
                Peers::Server => {
                    self.process(package.source(), package.decode()).await?;
                }
            }
        }
    }

    async fn process(
        &mut self,
        source: SocketAddr,
        messages: MessageDecoder<'_, ToServer>,
    ) -> anyhow::Result<()> {
        for message_result in messages {
            let Ok(message) = message_result else {
                warn!("Invalid package received");
                return Ok(());
            };

            match message {
                ToServer::Ping(id) => self.reply(&FromServer::Pong(id), source).await?,
                ToServer::OpenGame { max_players } => self.open_game(source, max_players).await?,
            }
        }

        Ok(())
    }

    async fn open_game(&mut self, source: SocketAddr, max_players: u8) -> anyhow::Result<()> {
        match Socket::bind(None).await {
            Ok(socket) => {
                let port = socket.port();
                info!("Starting new game on port {port}.");
                self.reply(&FromServer::GameOpened { port }, source).await?;
                game::startup(socket, source, max_players).await;
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
