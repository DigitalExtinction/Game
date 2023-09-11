use bevy::prelude::*;
use de_core::{
    assets::asset_path,
    gconfig::{GameConfig, LocalPlayers},
    state::AppState,
};
use de_gui::ToastEvent;
use de_lobby_client::GetGameRequest;
use de_lobby_model::GameMap;
use de_map::hash::MapHash;
use de_messages::Readiness;
use de_multiplayer::{
    GameReadinessEvent, PeerJoinedEvent, PeerLeftEvent, ShutdownMultiplayerEvent,
};
use de_types::player::Player;

use super::ui::RefreshPlayersEvent;
use crate::multiplayer::{
    current::GameNameRes,
    requests::{Receiver, Sender},
    MultiplayerState,
};

pub(super) struct JoinedGameStatePlugin;

impl Plugin for JoinedGameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGameEvent>()
            .add_systems(OnEnter(MultiplayerState::GameJoined), (setup, refresh))
            .add_systems(OnExit(MultiplayerState::GameJoined), cleanup)
            .add_systems(
                Update,
                (
                    refresh
                        .run_if(on_event::<PeerJoinedEvent>().or_else(on_event::<PeerLeftEvent>())),
                    handle_get_response,
                    start
                        .run_if(on_event::<StartGameEvent>())
                        .after(handle_get_response),
                    handle_readiness,
                )
                    .run_if(in_state(MultiplayerState::GameJoined)),
            );
    }
}

#[derive(Event)]
struct StartGameEvent(GameMap);

#[derive(Resource)]
pub(crate) struct LocalPlayerRes(Player);

impl LocalPlayerRes {
    pub(crate) fn new(player: Player) -> Self {
        Self(player)
    }
}

#[derive(Resource)]
struct ReadyRes(bool);

fn setup(mut commands: Commands) {
    commands.insert_resource(ReadyRes(false));
}

fn cleanup(
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut shutdown: EventWriter<ShutdownMultiplayerEvent>,
) {
    commands.remove_resource::<LocalPlayerRes>();
    commands.remove_resource::<ReadyRes>();

    if state.as_ref() != &AppState::InGame {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn refresh(game_name: Res<GameNameRes>, mut sender: Sender<GetGameRequest>) {
    info!("Refreshing game info...");
    sender.send(GetGameRequest::new(game_name.name_owned()));
}

fn handle_readiness(
    mut events: EventReader<GameReadinessEvent>,
    game_name: Res<GameNameRes>,
    mut sender: Sender<GetGameRequest>,
    mut ready: ResMut<ReadyRes>,
) {
    if events.iter().all(|e| **e != Readiness::Ready) {
        return;
    }

    sender.send(GetGameRequest::new(game_name.name_owned()));
    ready.0 = true;
}

fn handle_get_response(
    mut multi_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<GetGameRequest>,
    mut start_events: EventWriter<StartGameEvent>,
    ready: Res<ReadyRes>,
    mut refresh: EventWriter<RefreshPlayersEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(game) => {
                refresh.send(RefreshPlayersEvent::from_slice(game.players()));

                if ready.0 {
                    start_events.send(StartGameEvent(game.setup().config().map().clone()));
                }
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                multi_state.set(MultiplayerState::SignIn);
            }
        }
    }
}

fn start(
    mut commands: Commands,
    mut events: EventReader<StartGameEvent>,
    player: Res<LocalPlayerRes>,
    mut app_state: ResMut<NextState<AppState>>,
    mut multi_state: ResMut<NextState<MultiplayerState>>,
    mut toasts: EventWriter<ToastEvent>,
) {
    let Some(event) = events.iter().last() else {
        return;
    };

    let map_path = match MapHash::from_hex(event.0.hash()) {
        Ok(hash) => hash.construct_path(asset_path("maps")),
        Err(error) => {
            toasts.send(ToastEvent::new(error));
            multi_state.set(MultiplayerState::SignIn);
            return;
        }
    };

    commands.insert_resource(GameConfig::new(
        map_path,
        true,
        LocalPlayers::from_single(player.0),
    ));
    app_state.set(AppState::InGame);
}
