use bevy::prelude::*;
use de_core::state::AppState;
use de_multiplayer::ShutdownMultiplayerEvent;

use super::MultiplayerState;

pub(crate) struct JoinedGamePlugin;

impl Plugin for JoinedGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(MultiplayerState::GameJoined), cleanup);
    }
}

fn cleanup(state: Res<State<AppState>>, mut shutdown: EventWriter<ShutdownMultiplayerEvent>) {
    if state.as_ref() != &AppState::InGame {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}
