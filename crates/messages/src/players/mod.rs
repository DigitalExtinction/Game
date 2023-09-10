use bincode::{Decode, Encode};
pub use chat::{ChatMessage, ChatMessageError, MAX_CHAT_LEN};

mod chat;

/// Messages to be sent by a player/client or occasionally the game server to
/// other players.
#[derive(Debug, Decode)]
pub struct FromPlayers {
    /// ID of the sending player.
    source: u8,
    /// Original message.
    message: ToPlayers,
}

impl FromPlayers {
    /// ID of the sending player
    pub fn source(&self) -> u8 {
        self.source
    }

    pub fn message(&self) -> &ToPlayers {
        &self.message
    }
}

/// Messages to be sent by a player/client or occasionally the game server to
/// other players.
#[derive(Debug, Encode, Clone, Copy)]
pub struct BorrowedFromPlayers<'a> {
    /// ID of the sending player.
    source: u8,
    /// Original message.
    message: &'a ToPlayers,
}

impl<'a> BorrowedFromPlayers<'a> {
    pub fn new(source: u8, message: &'a ToPlayers) -> Self {
        Self { source, message }
    }
}

/// Message to be sent by a player/client or occasionally the game server to
/// the game server for the distribution to other game players.
#[derive(Debug, Encode, Decode)]
pub enum ToPlayers {
    Chat(ChatMessage),
}
