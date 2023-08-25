use std::{net::SocketAddr, time::Instant};

use bevy::prelude::*;
use de_core::schedule::PreMovement;
use de_messages::{FromGame, FromServer, ToGame, ToServer};
use de_net::{InPackage, PackageBuilder, Peers, Reliability};

use crate::{
    config::ConnectionType,
    lifecycle::{FatalErrorEvent, NetGameConfRes},
    netstate::NetState,
    network::{NetworkSet, PackageReceivedEvent, SendPackageEvent},
};

pub(crate) struct MessagesPlugin;

impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToMainServerEvent>()
            .add_event::<ToGameServerEvent>()
            .add_event::<FromMainServerEvent>()
            .add_event::<FromGameServerEvent>()
            .add_systems(OnEnter(NetState::Connecting), setup)
            .add_systems(OnEnter(NetState::None), cleanup)
            .add_systems(
                PostUpdate,
                (
                    message_sender::<ToMainServerEvent>
                        .run_if(on_event::<ToMainServerEvent>())
                        .in_set(MessagesSet::SendMessages)
                        .before(NetworkSet::SendPackages),
                    message_sender::<ToGameServerEvent>
                        .run_if(on_event::<ToGameServerEvent>())
                        .in_set(MessagesSet::SendMessages)
                        .before(NetworkSet::SendPackages),
                ),
            )
            .add_systems(
                PreMovement,
                recv_messages
                    .run_if(on_event::<PackageReceivedEvent>())
                    .in_set(MessagesSet::RecvMessages)
                    .after(NetworkSet::RecvPackages),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum MessagesSet {
    SendMessages,
    RecvMessages,
}

trait ToMessage
where
    Self: Event,
{
    type Message: bincode::Encode;
    const PORT_TYPE: PortType;

    fn reliability(&self) -> Reliability;
    fn message(&self) -> &Self::Message;
}

#[derive(Event)]
pub(crate) struct ToMainServerEvent(ToServer);

impl From<ToServer> for ToMainServerEvent {
    fn from(message: ToServer) -> Self {
        Self(message)
    }
}

impl ToMessage for ToMainServerEvent {
    type Message = ToServer;
    const PORT_TYPE: PortType = PortType::Main;

    fn reliability(&self) -> Reliability {
        Reliability::SemiOrdered
    }

    fn message(&self) -> &Self::Message {
        &self.0
    }
}

#[derive(Event)]
pub(crate) struct ToGameServerEvent {
    reliability: Reliability,
    message: ToGame,
}

impl ToGameServerEvent {
    pub(crate) fn new(reliability: Reliability, message: ToGame) -> Self {
        Self {
            reliability,
            message,
        }
    }
}

impl ToMessage for ToGameServerEvent {
    type Message = ToGame;
    const PORT_TYPE: PortType = PortType::Game;

    fn reliability(&self) -> Reliability {
        self.reliability
    }

    fn message(&self) -> &Self::Message {
        &self.message
    }
}

trait InMessageEvent
where
    Self: Event,
{
    type M;

    fn from_message(time: Instant, message: Self::M) -> Self;
}

#[derive(Event)]
pub(crate) struct FromMainServerEvent(FromServer);

impl FromMainServerEvent {
    pub(crate) fn message(&self) -> &FromServer {
        &self.0
    }
}

impl InMessageEvent for FromMainServerEvent {
    type M = FromServer;

    fn from_message(_time: Instant, message: Self::M) -> Self {
        Self(message)
    }
}

#[derive(Event)]
pub(crate) struct FromGameServerEvent {
    time: Instant,
    message: FromGame,
}

impl FromGameServerEvent {
    pub(crate) fn time(&self) -> Instant {
        self.time
    }

    pub(crate) fn message(&self) -> &FromGame {
        &self.message
    }
}

impl InMessageEvent for FromGameServerEvent {
    type M = FromGame;

    fn from_message(time: Instant, message: Self::M) -> Self {
        Self { time, message }
    }
}

/// Already known ports of the main and game server.
#[derive(Resource)]
pub(crate) enum Ports {
    Main(u16),
    Game(u16),
    Both { main: u16, game: u16 },
}

impl Ports {
    /// The game port is stored if it is not yet known. Otherwise, the new port
    /// is compared to the existing one. If they do not match, an error is
    /// returned.
    pub(crate) fn init_game_port(&mut self, port: u16) -> Result<(), String> {
        match self {
            Self::Main(main) => {
                *self = Self::Both {
                    main: *main,
                    game: port,
                };

                Ok(())
            }
            Self::Both { game, .. } | Self::Game(game) => {
                if port == *game {
                    Ok(())
                } else {
                    Err(format!("Game change game port ({} -> {}).", *game, port))
                }
            }
        }
    }

    fn port(&self, port_type: PortType) -> Option<u16> {
        match port_type {
            PortType::Game => self.game(),
            PortType::Main => self.main(),
        }
    }

    /// Returns port of the main server if known.
    fn main(&self) -> Option<u16> {
        match self {
            Self::Main(port) => Some(*port),
            Self::Both { main, .. } => Some(*main),
            Self::Game(_) => None,
        }
    }

    /// Returns port of the game server if known.
    fn game(&self) -> Option<u16> {
        match self {
            Self::Game(port) => Some(*port),
            Self::Both { game, .. } => Some(*game),
            Self::Main(_) => None,
        }
    }

    /// Returns true if `port` corresponds to the port of the main server.
    fn is_main(&self, port: u16) -> bool {
        self.main().map_or(false, |p| p == port)
    }
}

impl From<ConnectionType> for Ports {
    fn from(game_type: ConnectionType) -> Self {
        match game_type {
            ConnectionType::CreateGame { port, .. } => Self::Main(port),
            ConnectionType::JoinGame(port) => Self::Game(port),
        }
    }
}

#[derive(Clone, Copy)]
enum PortType {
    Main,
    Game,
}

fn setup(mut commands: Commands, conf: Res<NetGameConfRes>) {
    let ports: Ports = conf.connection_type().into();
    commands.insert_resource(ports);
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Ports>();
}

fn message_sender<E>(
    conf: Res<NetGameConfRes>,
    ports: Res<Ports>,
    mut inputs: EventReader<E>,
    mut outputs: EventWriter<SendPackageEvent>,
) where
    E: ToMessage,
{
    let Some(port) = ports.port(E::PORT_TYPE) else {
        warn!("Port not (yet) known.");
        return;
    };
    let addr = SocketAddr::new(conf.server_host(), port);

    let mut unreliable = PackageBuilder::new(Reliability::Unreliable, Peers::Server, addr);
    let mut unordered = PackageBuilder::new(Reliability::Unordered, Peers::Server, addr);
    let mut semi_ordered = PackageBuilder::new(Reliability::SemiOrdered, Peers::Server, addr);

    for event in inputs.iter() {
        let builder = match event.reliability() {
            Reliability::Unreliable => &mut unreliable,
            Reliability::Unordered => &mut unordered,
            Reliability::SemiOrdered => &mut semi_ordered,
        };
        builder.push(event.message()).unwrap();
    }

    for builder in [unreliable, unordered, semi_ordered] {
        for package in builder.build() {
            outputs.send(package.into());
        }
    }
}

fn recv_messages(
    ports: Res<Ports>,
    mut packages: EventReader<PackageReceivedEvent>,
    mut main_server: EventWriter<FromMainServerEvent>,
    mut game_server: EventWriter<FromGameServerEvent>,
    mut fatals: EventWriter<FatalErrorEvent>,
) {
    for event in packages.iter() {
        let package = event.package();
        if ports.is_main(package.source().port()) {
            decode_and_send::<FromServer, _>(package, &mut main_server, &mut fatals);
        } else {
            decode_and_send::<FromGame, _>(package, &mut game_server, &mut fatals);
        }
    }
}

fn decode_and_send<P, E>(
    package: &InPackage,
    events: &mut EventWriter<E>,
    fatals: &mut EventWriter<FatalErrorEvent>,
) where
    P: bincode::Decode,
    E: InMessageEvent<M = P>,
{
    for message in package.decode::<P>() {
        match message {
            Ok(message) => {
                events.send(E::from_message(package.time(), message));
            }
            Err(err) => {
                fatals.send(FatalErrorEvent::new(format!(
                    "Invalid data received: {:?}",
                    err
                )));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use de_core::player::Player;

    use super::*;

    #[test]
    fn test_ports() {
        let mut ports = Ports::from(ConnectionType::CreateGame {
            port: 2,
            max_players: Player::Player1,
        });
        assert_eq!(ports.main(), Some(2));
        assert_eq!(ports.game(), None);
        ports.init_game_port(3).unwrap();
        assert_eq!(ports.main(), Some(2));
        assert_eq!(ports.game(), Some(3));

        let mut ports = Ports::from(ConnectionType::JoinGame(4));
        assert_eq!(ports.main(), None);
        assert_eq!(ports.game(), Some(4));
        ports.init_game_port(4).unwrap();
        assert!(ports.init_game_port(5).is_err());
    }
}
