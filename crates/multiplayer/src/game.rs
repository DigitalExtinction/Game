use std::net::SocketAddr;

use bevy::prelude::*;
use de_core::{player::Player, schedule::PreMovement};
use de_messages::{FromGame, FromServer, GameOpenError, JoinError, Readiness, ToGame, ToServer};
use de_net::Reliability;

use crate::{
    config::ConnectionType,
    lifecycle::{FatalErrorEvent, NetGameConfRes},
    messages::{
        FromGameServerEvent, FromMainServerEvent, MessagesSet, Ports, ToGameServerEvent,
        ToMainServerEvent,
    },
    netstate::NetState,
};

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameOpenedEvent>()
            .add_event::<GameJoinedEvent>()
            .add_event::<PeerJoinedEvent>()
            .add_event::<PeerLeftEvent>()
            .add_event::<GameReadinessEvent>()
            .add_event::<SetReadinessEvent>()
            .add_systems(OnEnter(NetState::Connected), open_or_join)
            .add_systems(
                PreMovement,
                (
                    process_from_server
                        .run_if(on_event::<FromMainServerEvent>())
                        .after(MessagesSet::RecvMessages),
                    process_from_game
                        .run_if(on_event::<FromGameServerEvent>())
                        .after(MessagesSet::RecvMessages),
                ),
            )
            .add_systems(
                PostUpdate,
                set_readiness
                    .run_if(in_state(NetState::Joined))
                    .run_if(on_event::<SetReadinessEvent>())
                    .before(MessagesSet::SendMessages),
            )
            .add_systems(OnEnter(NetState::ShuttingDown), leave);
    }
}

/// A new game on the given socket address was just opened.
#[derive(Event)]
pub struct GameOpenedEvent(pub SocketAddr);

/// A game was just joined.
#[derive(Event)]
pub struct GameJoinedEvent {
    player: Player,
}

impl GameJoinedEvent {
    fn new(player: Player) -> Self {
        Self { player }
    }

    pub fn player(&self) -> Player {
        self.player
    }
}

#[derive(Event)]
pub struct PeerJoinedEvent(u8);

impl PeerJoinedEvent {
    pub fn id(&self) -> u8 {
        self.0
    }
}

#[derive(Event)]
pub struct PeerLeftEvent(u8);

impl PeerLeftEvent {
    pub fn id(&self) -> u8 {
        self.0
    }
}

/// This event is sent when game readiness stage of the joined game changes.
#[derive(Event, Deref)]
pub struct GameReadinessEvent(Readiness);

/// Send this event to change player readiness stage.
#[derive(Event)]
pub struct SetReadinessEvent(Readiness);

impl From<Readiness> for SetReadinessEvent {
    fn from(readiness: Readiness) -> Self {
        Self(readiness)
    }
}

fn open_or_join(
    conf: Res<NetGameConfRes>,
    mut main_server: EventWriter<ToMainServerEvent>,
    mut game_server: EventWriter<ToGameServerEvent>,
) {
    match conf.connection_type() {
        ConnectionType::CreateGame { max_players, .. } => {
            info!("Sending a open-game request.");
            main_server.send(
                ToServer::OpenGame {
                    max_players: max_players.to_num(),
                }
                .into(),
            );
        }
        ConnectionType::JoinGame(_) => {
            info!("Sending a join-game request.");
            game_server.send(ToGameServerEvent::new(
                Reliability::SemiOrdered,
                ToGame::Join,
            ));
        }
    }
}

fn process_from_server(
    conf: Res<NetGameConfRes>,
    mut ports: ResMut<Ports>,
    mut events: EventReader<FromMainServerEvent>,
    mut outputs: EventWriter<ToGameServerEvent>,
    mut opened: EventWriter<GameOpenedEvent>,
    mut fatals: EventWriter<FatalErrorEvent>,
) {
    for event in events.iter() {
        match event.message() {
            FromServer::Pong(id) => {
                info!("Pong {} received from server.", *id);
            }
            FromServer::GameOpened { port } => match ports.init_game_port(*port) {
                Ok(_) => {
                    info!("Game on port {} opened.", *port);
                    // Send something to open NAT.
                    outputs.send(ToGameServerEvent::new(
                        Reliability::Unordered,
                        ToGame::Ping(u32::MAX),
                    ));
                    opened.send(GameOpenedEvent(SocketAddr::new(conf.server_host(), *port)));
                }
                Err(err) => {
                    fatals.send(FatalErrorEvent::new(format!("Invalid GameOpened: {err:?}")));
                }
            },
            FromServer::GameOpenError(err) => match err {
                GameOpenError::DifferentGame => {
                    fatals.send(FatalErrorEvent::new(
                        "Cannot open game, the player already joined a game.",
                    ));
                }
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_from_game(
    mut inputs: EventReader<FromGameServerEvent>,
    mut fatals: EventWriter<FatalErrorEvent>,
    state: Res<State<NetState>>,
    mut joined_events: EventWriter<GameJoinedEvent>,
    mut peer_joined_events: EventWriter<PeerJoinedEvent>,
    mut peer_left_events: EventWriter<PeerLeftEvent>,
    mut readiness_events: EventWriter<GameReadinessEvent>,
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
                    info!("Joined game as Player {player}.");
                    next_state.set(NetState::Joined);
                    joined_events.send(GameJoinedEvent::new(player));
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
                JoinError::GameNotOpened => {
                    fatals.send(FatalErrorEvent::new(
                        "Game is no longer opened, cannot join.",
                    ));
                }
                JoinError::AlreadyJoined => {
                    fatals.send(FatalErrorEvent::new(
                        "Already joined the game, cannot re-join.",
                    ));
                }
                JoinError::DifferentGame => {
                    fatals.send(FatalErrorEvent::new(
                        "Player already joined a different game.",
                    ));
                }
            },
            FromGame::Left => {
                if state.get() < &NetState::ShuttingDown {
                    fatals.send(FatalErrorEvent::new("Player was kicked from the game."));
                }
            }
            FromGame::PeerJoined(id) => {
                info!("Peer {id} joined.");
                peer_joined_events.send(PeerJoinedEvent(*id));
            }
            FromGame::PeerLeft(id) => {
                info!("Peer {id} left.");
                peer_left_events.send(PeerLeftEvent(*id));
            }
            FromGame::GameReadiness(readiness) => {
                info!("Game readiness changed to: {readiness:?}");
                readiness_events.send(GameReadinessEvent(*readiness));
            }
        }
    }
}

fn set_readiness(
    mut readiness_events: EventReader<SetReadinessEvent>,
    mut message_events: EventWriter<ToGameServerEvent>,
) {
    let Some(readiness) = readiness_events.iter().last() else {
        return;
    };

    message_events.send(ToGameServerEvent::new(
        Reliability::SemiOrdered,
        ToGame::Readiness(readiness.0),
    ));
}

fn leave(mut server: EventWriter<ToGameServerEvent>) {
    info!("Sending leave game message.");
    // Send this even if not yet joined because the join / open-game request
    // might already be processed.
    server.send(ToGameServerEvent::new(
        Reliability::SemiOrdered,
        ToGame::Leave,
    ));
}
