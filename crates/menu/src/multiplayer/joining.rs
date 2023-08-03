use std::net::SocketAddr;

use bevy::prelude::*;
use de_gui::ToastEvent;
use de_lobby_client::{GetGameRequest, JoinGameRequest};
use de_multiplayer::{
    ConnectionType, GameJoinedEvent, NetGameConf, ShutdownMultiplayerEvent, StartMultiplayerEvent,
};

use super::{
    requests::{Receiver, RequestsPlugin, Sender},
    MultiplayerState,
};
use crate::MenuState;

pub(crate) struct JoiningGamePlugin;

impl Plugin for JoiningGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RequestsPlugin::<GetGameRequest>::new(),
            RequestsPlugin::<JoinGameRequest>::new(),
        ))
        .add_event::<JoinGameEvent>()
        .add_systems(OnEnter(MultiplayerState::GameJoining), get_game)
        .add_systems(OnExit(MultiplayerState::GameJoining), cleanup)
        .add_systems(
            PreUpdate,
            handle_join_event.run_if(in_state(MenuState::Multiplayer)),
        )
        .add_systems(
            Update,
            (
                handle_get_response,
                handle_join_response.run_if(resource_exists::<GameSocketAddrRes>()),
                handle_joined_event.run_if(on_event::<GameJoinedEvent>()),
            )
                .run_if(in_state(MultiplayerState::GameJoining)),
        );
    }
}

/// Send this event to initiate joining of a multiplayer game.
///
/// The game will be joined at both DE Lobby and the DE Connector. Once this is
/// done, the menu transitions to [`MultiplayerState::GameJoined`].
#[derive(Event)]
pub(super) struct JoinGameEvent(String);

impl JoinGameEvent {
    pub(super) fn new(name: String) -> Self {
        Self(name)
    }
}

#[derive(Resource)]
pub(crate) struct GameNameRes(String);

#[derive(Resource)]
pub(crate) struct GameSocketAddrRes(SocketAddr);

fn handle_join_event(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut events: EventReader<JoinGameEvent>,
) {
    let Some(event) = events.iter().last() else {
        return;
    };

    commands.insert_resource(GameNameRes(event.0.to_owned()));
    next_state.set(MultiplayerState::GameJoining);
}

fn cleanup(
    mut commands: Commands,
    state: Res<State<MultiplayerState>>,
    mut shutdown: EventWriter<ShutdownMultiplayerEvent>,
) {
    commands.remove_resource::<GameNameRes>();
    commands.remove_resource::<GameSocketAddrRes>();

    if state.as_ref() != &MultiplayerState::GameJoined {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn get_game(game_name: Res<GameNameRes>, mut sender: Sender<GetGameRequest>) {
    sender.send(GetGameRequest::new(game_name.0.to_owned()));
}

fn handle_get_response(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<GetGameRequest>,
    mut sender: Sender<JoinGameRequest>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(game) => {
                sender.send(JoinGameRequest::new(
                    game.setup().config().name().to_owned(),
                ));
                commands.insert_resource(GameSocketAddrRes(game.setup().server()));
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                next_state.set(MultiplayerState::SignIn);
            }
        }
    }
}

fn handle_join_response(
    server: Res<GameSocketAddrRes>,
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<JoinGameRequest>,
    mut multiplayer: EventWriter<StartMultiplayerEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(_) => {
                multiplayer.send(StartMultiplayerEvent::new(NetGameConf::new(
                    server.0.ip(),
                    ConnectionType::JoinGame(server.0.port()),
                )));
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                next_state.set(MultiplayerState::SignIn);
            }
        }
    }
}

fn handle_joined_event(mut next_state: ResMut<NextState<MultiplayerState>>) {
    next_state.set(MultiplayerState::GameJoined);
}
