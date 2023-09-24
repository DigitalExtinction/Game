use std::net::SocketAddr;

use anyhow::Context;
use async_std::task;
use de_messages::{FromServer, GameOpenError, ToServer};
use de_net::{
    self, MessageDecoder, OutPackage, PackageReceiver, PackageSender, Peers, Reliability, Socket,
};
use de_types::player::Player;
use tracing::{error, info, warn};

use crate::{clients::Clients, game};

/// Main game server responsible for initial communication with clients and
/// establishment of game sub-servers.
pub(crate) struct MainServer {
    outputs: PackageSender,
    inputs: PackageReceiver,
    clients: Clients,
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
        Self {
            outputs,
            inputs,
            clients: Clients::new(),
        }
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

    async fn open_game(&mut self, source: SocketAddr, max_players: Player) -> anyhow::Result<()> {
        if let Err(err) = self.clients.reserve(source).await {
            warn!("OpenGame request error: {err}");
            self.reply(
                &FromServer::GameOpenError(GameOpenError::DifferentGame),
                source,
            )
            .await?;
            return Ok(());
        }

        match Socket::bind(None).await {
            Ok(socket) => {
                let port = socket.port();
                self.clients.set(source, port).await;

                info!("Starting new game on port {port}.");
                self.reply(&FromServer::GameOpened { port }, source).await?;
                game::startup(self.clients.clone(), socket, source, max_players).await;
                Ok(())
            }
            Err(error) => {
                error!("Failed to open a new game: {:?}", error);
                self.clients.free(source).await;
                Ok(())
            }
        }
    }

    async fn reply(&mut self, message: &FromServer, target: SocketAddr) -> anyhow::Result<()> {
        self.outputs
            .send(
                OutPackage::encode_single(message, Reliability::Unordered, Peers::Server, target)
                    .unwrap(),
            )
            .await
            .context("Failed to send a reply")
    }
}
