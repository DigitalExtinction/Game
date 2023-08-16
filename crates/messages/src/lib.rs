//! This crate implements messages to be exchanged among players and DE
//! Connector during multiplayer game.

pub use game::{FromGame, JoinError, Readiness, ToGame};
pub use server::{FromServer, GameOpenError, ToServer};

mod game;
mod server;
