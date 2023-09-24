use bincode::{Decode, Encode};
use de_types::player::Player;

/// Message to be sent from a player/client to a main server (outside of a
/// game).
#[derive(Debug, Encode, Decode)]
pub enum ToServer {
    /// Prompts the server to respond [`FromServer::Pong`] with the same ping ID.
    Ping(u32),
    /// This message opens a new game on the server. The server responds with
    /// [`FromServer::GameOpened`].
    OpenGame { max_players: Player },
}

/// Message to be sent from a main server to a player/client (outside of a
/// game).
#[derive(Debug, Encode, Decode)]
pub enum FromServer {
    /// Response to [`ToServer::Ping`].
    Pong(u32),
    /// A new game was opened upon request from the client.
    GameOpened {
        /// Port at which players may connect to join the game.
        port: u16,
    },
    GameOpenError(GameOpenError),
}

#[derive(Debug, Encode, Decode)]
pub enum GameOpenError {
    /// The player opening the game has already joined a different game.
    DifferentGame,
}
