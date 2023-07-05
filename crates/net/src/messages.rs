use bincode::{Decode, Encode};

/// Message to be sent from a player/client to a main server (outside of a
/// game).
#[derive(Encode, Decode)]
pub enum ToServer {
    /// Prompts the server to respond [`FromServer::Pong`] with the same ping ID.
    Ping(u32),
    /// This message opens a new game on the server. The server responds with
    /// [`FromServer::GameOpened`].
    OpenGame { max_players: u8 },
}

/// Message to be sent from a main server to a player/client (outside of a
/// game).
#[derive(Encode, Decode)]
pub enum FromServer {
    /// Response to [`ToServer::Ping`].
    Pong(u32),
    /// A new game was opened upon request from the client.
    GameOpened {
        /// Port at which players may connect to join the game.
        port: u16,
    },
}

/// Message to be sent from a player/client to a game server (inside of a
/// game).
#[derive(Encode, Decode)]
pub enum ToGame {
    /// Prompts the server to respond [`FromGame::Pong`] with the same ping ID.
    Ping(u32),
    /// Connect the player to the game.
    Join,
    /// Disconnect the player from the game.
    ///
    /// The game is automatically closed once all players disconnect.
    Leave,
}

/// Message to be sent from a game server to a player/client (inside of a
/// game).
#[derive(Encode, Decode)]
pub enum FromGame {
    /// Response to [`ToGame::Ping`].
    Pong(u32),
    /// Informs the player that the server has discarded one or more incoming
    /// messages (to any peer) due to the player not being part of the game.
    NotJoined,
    /// Informs the player that they were just connected to the game under the
    /// ID.
    Joined(u8),
    /// Informs the player that they were not connected to the game due to an
    /// error.
    JoinError(JoinError),
    /// Informs the player the they is no longer part of the game. They might
    /// has been kicked out of the game or left voluntarily.
    Left,
    /// Informs the player that another player just connected to the same game
    /// under the given ID.
    PeerJoined(u8),
    /// Informs the player that another player with the given ID just
    /// disconnected from the same game.
    PeerLeft(u8),
}

#[derive(Encode, Decode)]
pub enum JoinError {
    GameFull,
}
