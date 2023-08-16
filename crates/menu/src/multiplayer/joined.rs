use bevy::prelude::*;
use de_core::{
    assets::asset_path,
    gconfig::{GameConfig, LocalPlayers},
    player::Player,
    state::AppState,
};
use de_gui::ToastEvent;
use de_lobby_client::GetGameRequest;
use de_map::hash::MapHash;
use de_messages::Readiness;
use de_multiplayer::{GameReadinessEvent, ShutdownMultiplayerEvent};

use super::{
    current::GameNameRes,
    requests::{Receiver, Sender},
    MultiplayerState,
};

pub(crate) struct JoinedGamePlugin;

impl Plugin for JoinedGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(MultiplayerState::GameJoined), cleanup)
            .add_systems(
                Update,
                (refresh, handle_get_response).run_if(in_state(MultiplayerState::GameJoined)),
            );
    }
}

fn cleanup(state: Res<State<AppState>>, mut shutdown: EventWriter<ShutdownMultiplayerEvent>) {
    if state.as_ref() != &AppState::InGame {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn refresh(game_name: Res<GameNameRes>, mut sender: Sender<GetGameRequest>) {
    // TODO when player joins or first time
    sender.send(GetGameRequest::new(game_name.name_owned()));
}

fn handle_get_response(
    mut commands: Commands,
    mut multi_state: ResMut<NextState<MultiplayerState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut receiver: Receiver<GetGameRequest>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(game) => {
                // TODO update UI
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                multi_state.set(MultiplayerState::SignIn);
            }
        }
    }
}
