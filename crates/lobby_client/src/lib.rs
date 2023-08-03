//! This crate implements Bevy based client to the DE lobby server.
//!
//! Send [`RequestEvent`] events to make requests and read [`ResponseEvent`]
//! events to receive request responses.
//!
//! The client is automatically authenticated when [`de_lobby_model::Token`]
//! response is received from any endpoint, thus it is sufficient to send
//! [`RequestEvent<SignInRequest>`] or [`RequestEvent<SignUpRequest>`].
//!
//! Use [`Authentication`] resource to obtain current authentication state and
//! detect its changes.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use client::Authentication;
pub use endpoints::*;
use plugin::EndpointPlugin;
pub use plugin::{RequestEvent, ResponseEvent, Result};
pub use requestable::LobbyRequest;
use systems::LobbyPlugin;

mod client;
mod endpoints;
mod plugin;
mod requestable;
mod systems;

pub struct LobbyClientPluginGroup;

impl PluginGroup for LobbyClientPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LobbyPlugin)
            .add(EndpointPlugin::<SignUpRequest>::default())
            .add(EndpointPlugin::<SignInRequest>::default())
            .add(EndpointPlugin::<CreateGameRequest>::default())
            .add(EndpointPlugin::<ListGamesRequest>::default())
            .add(EndpointPlugin::<GetGameRequest>::default())
            .add(EndpointPlugin::<JoinGameRequest>::default())
            .add(EndpointPlugin::<LeaveGameRequest>::default())
    }
}
