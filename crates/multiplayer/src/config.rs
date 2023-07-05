use std::net::IpAddr;

use de_core::player::Player;

pub struct NetGameConf {
    max_players: Player,
    server_host: IpAddr,
    server_port: ServerPort,
}

impl NetGameConf {
    pub fn new(max_players: Player, server_host: IpAddr, server_port: ServerPort) -> Self {
        Self {
            max_players,
            server_host,
            server_port,
        }
    }

    pub(crate) fn max_players(&self) -> Player {
        self.max_players
    }

    /// Address of DE Connector server.
    pub(crate) fn server_host(&self) -> IpAddr {
        self.server_host
    }

    pub(crate) fn server_port(&self) -> ServerPort {
        self.server_port
    }
}

#[derive(Clone, Copy)]
pub enum ServerPort {
    /// Port of a main server.
    ///
    /// This is not a game server thus the client must open a new game via this
    /// main server.
    Main(u16),
    /// Port of an existing game server.
    ///
    /// This is a game server with other players potentially already connected.
    Game(u16),
}
