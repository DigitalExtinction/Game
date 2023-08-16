use bincode::{Decode, Encode};

/// Message to be sent from a player/client to a game server (inside of a
/// game).
#[derive(Debug, Encode, Decode)]
pub enum ToGame {
    /// Prompts the server to respond [`FromGame::Pong`] with the same ping ID.
    Ping(u32),
    /// Connect the player to the game.
    Join,
    /// Disconnect the player from the game.
    ///
    /// The game is automatically closed once all players disconnect.
    Leave,
    /// Sets readiness of the client.
    ///
    /// New readiness must be greater by one or equal to the current readiness.
    /// See [`Readiness::progress`].
    Readiness(Readiness),
}

/// Message to be sent from a game server to a player/client (inside of a
/// game).
///
/// # Notes
///
/// * Players are numbered from 1 to `max_players` (inclusive).
#[derive(Debug, Encode, Decode)]
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
    /// Game readiness has changed.
    GameReadiness(Readiness),
}

#[derive(Debug, Encode, Decode)]
pub enum JoinError {
    GameFull,
    /// The game is no longer opened.
    GameNotOpened,
    /// The player has already joined the game.
    AlreadyJoined,
    /// The player already participates on a different game.
    DifferentGame,
}

/// Readiness of an individual client or the game as a whole. It consists of a
/// progression of individual variants / stages. Once all clients progress to a
/// readiness stage, the game progresses to that stage as well.
#[derive(Clone, Copy, Default, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub enum Readiness {
    /// Initial stage for all clients and the game.
    #[default]
    NotReady,
    /// The client / game is ready for the game to start.
    Ready,
    /// The client / game is prepared for game initialization to begin.
    Prepared,
    /// The client / game is ready for the game to start.
    ///
    /// The actually game-play happens in this readiness stage.
    Initialized,
}

impl Readiness {
    pub fn progress(self) -> Option<Self> {
        match self {
            Self::NotReady => Some(Self::Ready),
            Self::Ready => Some(Self::Prepared),
            Self::Prepared => Some(Self::Initialized),
            Self::Initialized => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readiness() {
        assert!(Readiness::default() < Readiness::Ready);
        assert!(Readiness::NotReady < Readiness::Ready);
        assert!(Readiness::Ready < Readiness::Prepared);
        assert!(Readiness::Prepared < Readiness::Initialized);
        assert!(Readiness::NotReady < Readiness::Initialized);

        assert_eq!(Readiness::NotReady.progress().unwrap(), Readiness::Ready);
    }
}
