use bevy::prelude::*;
use de_core::{baseset::GameSet, player::Player};
use de_net::{FromGame, FromServer, JoinError, ToGame, ToServer};

use crate::{
    lifecycle::{FatalErrorEvent, NetGameConfRes},
    messages::{
        FromGameServerEvent, FromMainServerEvent, MessagesSet, Ports, ToGameServerEvent,
        ToMainServerEvent,
    },
    netstate::NetState,
    ServerPort,
};

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(NetState::Connected)))
            .add_system(cleanup.in_schedule(OnEnter(NetState::None)))
            .add_system(open_or_join.in_schedule(OnEnter(NetState::Connected)))
            .add_system(
                process_game_opened
                    .in_base_set(GameSet::PreMovement)
                    .run_if(on_event::<FromMainServerEvent>())
                    .after(MessagesSet::RecvMessages),
            )
            .add_system(
                process_from_game
                    .in_base_set(GameSet::PreMovement)
                    .run_if(on_event::<FromGameServerEvent>())
                    .after(MessagesSet::RecvMessages),
            )
            .add_system(leave.in_schedule(OnEnter(NetState::ShuttingDown)));
    }
}

#[derive(Resource)]
pub(crate) struct Players {
    local: Option<Player>,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Players { local: None });
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Players>();
}

fn open_or_join(
    conf: Res<NetGameConfRes>,
    mut main_server: EventWriter<ToMainServerEvent>,
    mut game_server: EventWriter<ToGameServerEvent>,
) {
    match conf.server_port() {
        ServerPort::Main(_) => {
            info!("Sending a open-game request.");
            main_server.send(
                ToServer::OpenGame {
                    max_players: conf.max_players().to_num(),
                }
                .into(),
            );
        }
        ServerPort::Game(_) => {
            info!("Sending a join-game request.");
            game_server.send(ToGame::Join.into());
        }
    }
}

fn process_game_opened(
    mut ports: ResMut<Ports>,
    mut events: EventReader<FromMainServerEvent>,
    mut fatals: EventWriter<FatalErrorEvent>,
) {
    for event in events.iter() {
        if let FromServer::GameOpened { port } = event.message() {
            match ports.init_game_port(*port) {
                Ok(_) => {
                    info!("Game on port {} opened.", *port);
                }
                Err(err) => {
                    fatals.send(FatalErrorEvent::new(format!("Invalid GameOpened: {err:?}")));
                }
            }
        }
    }
}

fn process_from_game(
    mut players: ResMut<Players>,
    mut inputs: EventReader<FromGameServerEvent>,
    mut fatals: EventWriter<FatalErrorEvent>,
    state: Res<State<NetState>>,
    mut next_state: ResMut<NextState<NetState>>,
) {
    for event in inputs.iter() {
        match event.message() {
            FromGame::Pong(id) => {
                trace!("Received Pong({id}).");
            }
            FromGame::NotJoined => {
                fatals.send(FatalErrorEvent::new(
                    "Player is no longer part of the game.",
                ));
            }
            FromGame::Joined(id) => match Player::try_from(*id) {
                Ok(player) => {
                    players.local = Some(player);
                    next_state.set(NetState::Joined);
                }
                Err(err) => {
                    fatals.send(FatalErrorEvent::new(format!(
                        "Invalid player assigned by the server: {err:?}"
                    )));
                }
            },
            FromGame::JoinError(error) => match error {
                JoinError::GameFull => {
                    fatals.send(FatalErrorEvent::new("Game is full, cannot join."));
                }
            },
            FromGame::Left => {
                if state.0 < NetState::ShuttingDown {
                    fatals.send(FatalErrorEvent::new("Player was kicked from the game."));
                }
            }
            FromGame::PeerJoined(id) => {
                info!("Peer {id} joined.");
            }
            FromGame::PeerLeft(id) => {
                info!("Peer {id} left.");
            }
        }
    }
}

fn leave(mut server: EventWriter<ToGameServerEvent>) {
    // Send this even if not yet joined because the join / open-game request
    // might already be processed.
    server.send(ToGame::Leave.into());
}
