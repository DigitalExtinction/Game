use bevy::prelude::*;
use iyes_progress::ProgressPlugin;

use crate::MultiplayerGameConfig;

pub(super) struct NetStatePlugin;

impl Plugin for NetStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<NetState>()
            .add_plugin(ProgressPlugin::new(NetState::Connecting).continue_to(NetState::Connected))
            .add_system(setup.run_if(resource_added::<MultiplayerGameConfig>()))
            .add_system(cleanup.run_if(resource_removed::<MultiplayerGameConfig>()));
    }
}

/// Application state in regard to DE Connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum NetState {
    /// No connection to DE Connector was ever tried to be established.
    #[default]
    None,
    /// Connection to DE Connector is being bootstrapped.
    Connecting,
    /// Connection to DE Connector was just established.
    Connected,
    /// There was a connection or an attempt to establish a connection in the
    /// past.
    Disconnected,
}

fn setup(mut next_state: ResMut<NextState<NetState>>) {
    next_state.set(NetState::Connecting);
}

fn cleanup(current_state: Res<State<NetState>>, mut next_state: ResMut<NextState<NetState>>) {
    if current_state.0 == NetState::None || current_state.0 == NetState::Disconnected {
        return;
    }

    next_state.set(NetState::Disconnected);
}
