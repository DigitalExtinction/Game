use std::net::IpAddr;

pub struct NetGameConf {
    server_host: IpAddr,
    server_port: ServerPort,
}

impl NetGameConf {
    pub fn new(server_host: IpAddr, server_port: ServerPort) -> Self {
        Self {
            server_host,
            server_port,
        }
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
