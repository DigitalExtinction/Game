use bevy::prelude::*;
use iyes_progress::ProgressPlugin;

pub(super) struct NetStatePlugin;

impl Plugin for NetStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<NetState>().add_plugins((
            ProgressPlugin::new(NetState::Connecting).continue_to(NetState::Connected),
            ProgressPlugin::new(NetState::ShuttingDown).continue_to(NetState::None),
        ));
    }
}

/// Application state in regard to DE Connector.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Default, States)]
pub enum NetState {
    /// No connection to DE Connector is currently established.
    #[default]
    None,
    /// Connection to DE Connector is being bootstrapped.
    Connecting,
    /// Connection to DE Connector was just established.
    Connected,
    /// Client has joined a game.
    Joined,
    /// Multiplayer is being actively shut down.
    ShuttingDown,
}
