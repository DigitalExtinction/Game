use bincode::{Decode, Encode};

/// Message item to be sent from a player/client to a main server (outside of a
/// game).
#[derive(Encode, Decode)]
pub enum ToServer {
    /// This message opens a new game on the server. The server responds with
    /// [`FromServer::GameOpened`].
    OpenGame,
}

/// Message item to be sent from a main server to a player/client (outside of a
/// game).
#[derive(Encode, Decode)]
pub enum FromServer {
    /// A new game was opened upon request from the client.
    GameOpened {
        /// Port at which players may connect to join the game.
        port: u16,
    },
}

/// Message item to be sent from a player/client to a game server (inside of a
/// game).
#[derive(Encode, Decode)]
pub enum ToGame {
    /// Requests closure of the game.
    CloseGame,
    /// Prompts the server to respond [`FromGame::Pong`] with the same ping ID.
    Ping(u32),
}

/// Message item to be sent from a game server to a player/client (inside of a
/// game).
#[derive(Encode, Decode)]
pub enum FromGame {
    /// Informs the client that the game was closed and the game server will
    /// soon finish.
    GameClosed,
    /// Response to [`ToGame::Ping`].
    Pong(u32),
}
