use bevy::prelude::*;
use de_conf::Configuration;
use de_gui::ToastEvent;
use de_lobby_client::CreateGameRequest;
use de_lobby_model::{GameConfig, GameSetup};
use de_multiplayer::{
    ConnectionType, GameOpenedEvent, NetGameConf, ShutdownMultiplayerEvent, StartMultiplayerEvent,
};

use super::{
    current::GameNameRes,
    requests::{Receiver, Sender},
    MultiplayerState,
};
use crate::MenuState;

pub(crate) struct SetupGamePlugin;

impl Plugin for SetupGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetupGameEvent>()
            .add_systems(OnEnter(MultiplayerState::GameSetup), setup_network)
            .add_systems(OnExit(MultiplayerState::GameSetup), cleanup)
            .add_systems(
                PreUpdate,
                handle_setup_event.run_if(in_state(MenuState::Multiplayer)),
            )
            .add_systems(
                Update,
                (create_game_in_lobby, handle_lobby_response)
                    .run_if(in_state(MultiplayerState::GameSetup)),
            );
    }
}

/// Send this event to initiate new multiplayer setup.
///
/// The game will be opened at a DE Connector and registered at a DE Lobby.
/// Once this is done, the menu transitions to
/// [`MultiplayerState::GameJoined`].
#[derive(Event)]
pub(super) struct SetupGameEvent {
    config: GameConfig,
}

impl SetupGameEvent {
    pub(super) fn new(config: GameConfig) -> Self {
        Self { config }
    }
}

#[derive(Resource)]
pub(crate) struct GameConfigRes(GameConfig);

fn handle_setup_event(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut events: EventReader<SetupGameEvent>,
) {
    let Some(event) = events.iter().last() else {
        return;
    };

    commands.insert_resource(GameConfigRes(event.config.clone()));
    next_state.set(MultiplayerState::GameSetup);
}

fn cleanup(
    mut commands: Commands,
    state: Res<State<MultiplayerState>>,
    mut shutdown: EventWriter<ShutdownMultiplayerEvent>,
) {
    commands.remove_resource::<GameConfigRes>();

    if state.as_ref() != &MultiplayerState::GameJoined {
        shutdown.send(ShutdownMultiplayerEvent);
    }
}

fn setup_network(
    config: Res<Configuration>,
    game_config: Res<GameConfigRes>,
    mut multiplayer: EventWriter<StartMultiplayerEvent>,
) {
    let connector_conf = config.multiplayer().connector();
    let net_game_conf = NetGameConf::new(
        connector_conf.ip(),
        ConnectionType::CreateGame {
            port: connector_conf.port(),
            max_players: game_config.0.max_players().try_into().unwrap(),
        },
    );
    multiplayer.send(StartMultiplayerEvent::new(net_game_conf));
}

fn create_game_in_lobby(
    mut commands: Commands,
    config: Res<GameConfigRes>,
    mut opened_events: EventReader<GameOpenedEvent>,
    mut sender: Sender<CreateGameRequest>,
) {
    let Some(opened_event) = opened_events.iter().last() else {
        return;
    };

    let game_config = config.0.clone();
    commands.insert_resource(GameNameRes::new(game_config.name()));
    let game_setup = GameSetup::new(opened_event.0, game_config);
    sender.send(CreateGameRequest::new(game_setup));
}

fn handle_lobby_response(
    mut next_state: ResMut<NextState<MultiplayerState>>,
    mut receiver: Receiver<CreateGameRequest>,
    mut toasts: EventWriter<ToastEvent>,
) {
    while let Some(result) = receiver.receive() {
        match result {
            Ok(_) => {
                info!("Game successfully created.");
                next_state.set(MultiplayerState::GameJoined);
            }
            Err(error) => {
                toasts.send(ToastEvent::new(error));
                next_state.set(MultiplayerState::SignIn);
            }
        }
    }
}
