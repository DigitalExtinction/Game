//! This crate implements messages to be exchanged among players and DE
//! Connector during multiplayer game.

pub use game::{FromGame, JoinError, Readiness, ToGame};
pub use players::{
    BorrowedFromPlayers, ChatMessage, ChatMessageError, EntityNet, FromPlayers, HealthDelta,
    NetEntityIndex, NetProjectile, PathError, PathNet, ToPlayers, TransformNet, Vec2Net, Vec3Net,
    Vec4Net, MAX_CHAT_LEN,
};
pub use server::{FromServer, GameOpenError, ToServer};

mod game;
mod players;
mod server;
