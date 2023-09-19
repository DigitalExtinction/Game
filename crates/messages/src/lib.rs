//! This crate implements messages to be exchanged among players and DE
//! Connector during multiplayer game.

pub use game::{FromGame, JoinError, Readiness, ToGame};
pub use players::{
    BorrowedFromPlayers, ChatMessage, ChatMessageError, EntityNet, FromPlayers, HealthDelta,
    NetEntityIndex, ToPlayers, MAX_CHAT_LEN,
};
pub use server::{FromServer, GameOpenError, ToServer};

mod game;
mod players;
mod server;
