use bevy::prelude::*;
use de_core::state::AppState;
use de_gui::ToastEvent;
use de_lobby_client::GetGameRequest;
use de_multiplayer::{PeerJoinedEvent, PeerLeftEvent, ShutdownMultiplayerEvent};

use super::ui::RefreshPlayersEvent;
use crate::multiplayer::{
    current::GameNameRes,
    requests::{Receiver, Sender},
    MultiplayerState,
};

pub(super) struct JoinedGameStatePlugin;

impl Plugin for JoinedGameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MultiplayerState::GameJoined), refresh)
            .add_systems(OnExit(MultiplayerState::GameJoined), cleanup)
            .add_systems(
                Update,
                (
                    refresh
                        .run_if(on_event::<PeerJoinedEvent>().or_else(on_event::<PeerLeftEvent>())),
                    handle_get_response,
                )
                    .run_if(in_state(MultiplayerState::GameJoined)),
            );
    }
}

fn cleanup(state: Res<State<AppState>>, mut shutdown: EventWriter<ShutdownMultiplayerEvent>) {
    if state.as_ref() != &AppState::InGame {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn refresh(game_name: Res<GameNameRes>, mut sender: Sender<GetGameRequest>) {
    info!("Refreshing game info...");
    sender.send(GetGameRequest::new(game_name.name_owned()));
}

fn handle_get_response(
    mut multi_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<GetGameRequest>,
    mut refresh: EventWriter<RefreshPlayersEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(game) => {
                refresh.send(RefreshPlayersEvent::from_slice(game.players()));
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                multi_state.set(MultiplayerState::SignIn);
            }
        }
    }
}
