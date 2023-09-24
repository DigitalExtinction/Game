use bevy::prelude::*;
use de_gui::ToastEvent;
use de_lobby_client::{GetGameRequest, JoinGameRequest};
use de_lobby_model::GamePlayerInfo;
use de_multiplayer::{
    ConnectionType, GameJoinedEvent, NetGameConf, ShutdownMultiplayerEvent, StartMultiplayerEvent,
};

use super::{
    current::GameNameRes,
    joined::LocalPlayerRes,
    requests::{Receiver, Sender},
    MultiplayerState,
};

pub(crate) struct JoiningGamePlugin;

impl Plugin for JoiningGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MultiplayerState::GameJoining), get_game)
            .add_systems(OnExit(MultiplayerState::GameJoining), cleanup)
            .add_systems(
                Update,
                (
                    handle_get_response,
                    handle_joined_event.run_if(on_event::<GameJoinedEvent>()),
                    handle_join_response,
                )
                    .run_if(in_state(MultiplayerState::GameJoining)),
            );
    }
}

fn cleanup(
    mut commands: Commands,
    state: Res<State<MultiplayerState>>,
    mut shutdown: EventWriter<ShutdownMultiplayerEvent>,
) {
    if state.as_ref() != &MultiplayerState::GameJoined {
        commands.remove_resource::<LocalPlayerRes>();
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn get_game(game_name: Res<GameNameRes>, mut sender: Sender<GetGameRequest>) {
    sender.send(GetGameRequest::new(game_name.name_owned()));
}

fn handle_get_response(
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<GetGameRequest>,
    mut multiplayer: EventWriter<StartMultiplayerEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(game) => {
                let server = game.setup().server();
                multiplayer.send(StartMultiplayerEvent::new(NetGameConf::new(
                    server.ip(),
                    ConnectionType::JoinGame(server.port()),
                )));
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                next_state.set(MultiplayerState::SignIn);
            }
        }
    }
}

fn handle_joined_event(
    mut commands: Commands,
    game_name: Res<GameNameRes>,
    mut events: EventReader<GameJoinedEvent>,
    mut sender: Sender<JoinGameRequest>,
) {
    let Some(event) = events.iter().last() else {
        return;
    };

    commands.insert_resource(LocalPlayerRes::new(event.player()));
    sender.send(JoinGameRequest::new(
        game_name.name_owned(),
        GamePlayerInfo::new(event.player().to_num()),
    ));
}

fn handle_join_response(
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<JoinGameRequest>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(_) => {
                next_state.set(MultiplayerState::GameJoined);
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                next_state.set(MultiplayerState::SignIn);
            }
        }
    }
}
