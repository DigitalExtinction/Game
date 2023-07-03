use std::net::IpAddr;

use bevy::prelude::*;

/// Insert this resource before starting a multiplayer game.
///
/// After the resource is inserted, a connection to DE Connector is
/// established. The connection is dropped when the resource is removed.
#[derive(Resource)]
pub struct MultiplayerGameConfig {
    server_host: IpAddr,
    server_port: ServerPort,
}

impl MultiplayerGameConfig {
    pub fn new(server_host: IpAddr, server_port: ServerPort) -> Self {
        Self {
            server_host,
            server_port,
        }
    }

    /// Address of DE Connector server.
    pub fn server_host(&self) -> IpAddr {
        self.server_host
    }

    pub fn server_port(&self) -> ServerPort {
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
