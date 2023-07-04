use std::net::SocketAddr;

use bevy::prelude::*;
use de_core::baseset::GameSet;
use de_net::{FromGame, FromServer, InPackage, PackageBuilder, Peers, ToGame, ToServer};

use crate::{
    config::ServerPort,
    lifecycle::NetGameConfRes,
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
            .add_system(setup.in_schedule(OnEnter(NetState::Connecting)))
            .add_system(cleanup.in_schedule(OnEnter(NetState::None)))
            .add_system(
                message_sender::<ToMainServerEvent>
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(on_event::<ToMainServerEvent>())
                    .in_set(MessagesSet::SendMessages)
                    .before(NetworkSet::SendPackages),
            )
            .add_system(
                message_sender::<ToGameServerEvent>
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(on_event::<ToGameServerEvent>())
                    .in_set(MessagesSet::SendMessages)
                    .before(NetworkSet::SendPackages),
            )
            .add_system(
                recv_messages
                    .in_base_set(GameSet::PreMovement)
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
    Self: Send + Sync + 'static,
{
    type Message: bincode::Encode;
    const PORT_TYPE: PortType;

    fn message(&self) -> &Self::Message;
}

pub(crate) struct ToMainServerEvent(ToServer);

impl ToMessage for ToMainServerEvent {
    type Message = ToServer;
    const PORT_TYPE: PortType = PortType::Main;

    fn message(&self) -> &Self::Message {
        &self.0
    }
}

pub(crate) struct ToGameServerEvent(ToGame);

impl ToMessage for ToGameServerEvent {
    type Message = ToGame;
    const PORT_TYPE: PortType = PortType::Game;

    fn message(&self) -> &Self::Message {
        &self.0
    }
}

pub(crate) struct FromMainServerEvent(FromServer);

impl From<FromServer> for FromMainServerEvent {
    fn from(message: FromServer) -> Self {
        Self(message)
    }
}

pub(crate) struct FromGameServerEvent(FromGame);

impl From<FromGame> for FromGameServerEvent {
    fn from(message: FromGame) -> Self {
        Self(message)
    }
}

/// Already known ports of the main and game server.
#[derive(Resource)]
pub(crate) enum Ports {
    Main(u16),
    Game(u16),
}

impl Ports {
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
            Self::Game(_) => None,
        }
    }

    /// Returns port of the game server if known.
    fn game(&self) -> Option<u16> {
        match self {
            Self::Game(port) => Some(*port),
            Self::Main(_) => None,
        }
    }

    /// Returns true if `port` corresponds to the port of the main server.
    fn is_main(&self, port: u16) -> bool {
        self.main().map_or(false, |p| p == port)
    }
}

impl From<ServerPort> for Ports {
    fn from(port: ServerPort) -> Self {
        match port {
            ServerPort::Main(port) => Self::Main(port),
            ServerPort::Game(port) => Self::Game(port),
        }
    }
}

#[derive(Clone, Copy)]
enum PortType {
    Main,
    Game,
}

fn setup(mut commands: Commands, conf: Res<NetGameConfRes>) {
    let ports: Ports = conf.server_port().into();
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
    let port = ports.port(E::PORT_TYPE).expect("Port not (yet) known.");
    let addr = SocketAddr::new(conf.server_host(), port);
    let mut builder = PackageBuilder::new(true, Peers::Server, addr);

    for event in inputs.iter() {
        builder.push(event.message()).unwrap();
    }
    for package in builder.build() {
        outputs.send(package.into());
    }
}

fn recv_messages(
    ports: Res<Ports>,
    mut packages: EventReader<PackageReceivedEvent>,
    mut main_server: EventWriter<FromMainServerEvent>,
    mut game_server: EventWriter<FromGameServerEvent>,
) {
    for event in packages.iter() {
        let package = event.package();
        if ports.is_main(package.source().port()) {
            decode_and_send::<FromServer, _>(package, &mut main_server);
        } else {
            decode_and_send::<FromGame, _>(package, &mut game_server);
        }
    }
}

fn decode_and_send<P, E>(package: &InPackage, events: &mut EventWriter<E>)
where
    P: bincode::Decode,
    E: From<P> + Send + Sync + 'static,
{
    for message in package.decode::<P>() {
        match message {
            Ok(message) => {
                events.send(message.into());
            }
            Err(err) => {
                warn!("Invalid data received: {:?}", err);
                break;
            }
        }
    }
}
