use std::net::IpAddr;

use de_types::player::Player;

pub struct NetGameConf {
    server_host: IpAddr,
    connection_type: ConnectionType,
}

impl NetGameConf {
    pub fn new(server_host: IpAddr, connection_type: ConnectionType) -> Self {
        Self {
            server_host,
            connection_type,
        }
    }

    /// Address of DE Connector server.
    pub(crate) fn server_host(&self) -> IpAddr {
        self.server_host
    }

    pub(crate) fn connection_type(&self) -> ConnectionType {
        self.connection_type
    }
}

/// Type of to be established connection to DE Connector.
#[derive(Clone, Copy)]
pub enum ConnectionType {
    /// Create a new game via the given main server.
    ///
    /// This is not a game server thus the client must open a new game via this
    /// main server.
    CreateGame {
        /// Port of the main server.
        port: u16,
        /// Maximum number of players to be configured for the new game.
        max_players: Player,
    },
    /// Join a game server at the given port.
    ///
    /// This is a game server with other players potentially already connected.
    JoinGame(u16),
}
