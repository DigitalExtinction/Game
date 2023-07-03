use std::ops::Deref;

use async_std::channel::{TryRecvError, TrySendError};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::baseset::GameSet;
use de_net::{
    startup, ConnErrorReceiver, ConnectionError, InPackage, OutPackage, PackageReceiver,
    PackageSender, Socket,
};
use futures_lite::future;
use iyes_progress::prelude::*;

use crate::netstate::NetState;

const MAX_RECV_PER_UPDATE: usize = 100;

pub(crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SendPackageEvent>()
            .add_event::<PackageReceivedEvent>()
            .add_event::<ConnErrorEvent>()
            .add_system(setup.in_schedule(OnEnter(NetState::Connecting)))
            .add_system(cleanup.in_schedule(OnEnter(NetState::Disconnected)))
            .add_system(
                wait_for_network
                    .track_progress()
                    .run_if(resource_exists::<NetworkStartup>()),
            )
            .add_system(
                send_packages
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(resource_exists::<Sender>())
                    .run_if(on_event::<SendPackageEvent>())
                    .in_set(NetworkSet::SendPackages),
            )
            .add_system(
                recv_packages
                    .in_base_set(GameSet::PreMovement)
                    .run_if(resource_exists::<Receiver>())
                    .in_set(NetworkSet::RecvPackages),
            )
            .add_system(
                recv_errors
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(resource_exists::<Errors>())
                    .in_set(NetworkSet::RecvErrors),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum NetworkSet {
    SendPackages,
    RecvPackages,
    RecvErrors,
}

/// Send this event to send a package (data) over the network.
///
/// The network must be established before this events are sent. The events are
/// drained and thus it is expected that the events are received only by
/// [`self::send_packages`] system.
pub(crate) struct SendPackageEvent(OutPackage);

/// This event is sent any time a new package from any source is received.
pub(crate) struct PackageReceivedEvent(InPackage);

/// This event is sent any time a network error is detected.
pub(crate) struct ConnErrorEvent(pub(crate) ConnectionError);

#[derive(Resource)]
struct NetworkStartup(Task<(PackageSender, PackageReceiver, ConnErrorReceiver)>);

#[derive(Resource)]
struct Sender(PackageSender);

impl Deref for Sender {
    type Target = PackageSender;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource)]
struct Receiver(PackageReceiver);

impl Deref for Receiver {
    type Target = PackageReceiver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource)]
struct Errors(ConnErrorReceiver);

impl Deref for Errors {
    type Target = ConnErrorReceiver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn setup(mut commands: Commands) {
    let pool = IoTaskPool::get();
    let task = pool.spawn(async {
        let socket = Socket::bind(None).await.unwrap();
        startup(|t| pool.spawn(t).detach(), socket)
    });
    commands.insert_resource(NetworkStartup(task));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<NetworkStartup>();
    commands.remove_resource::<Sender>();
    commands.remove_resource::<Receiver>();
    commands.remove_resource::<Errors>();
}

fn wait_for_network(mut commands: Commands, mut task: ResMut<NetworkStartup>) -> Progress {
    let Some((sender, receiver, errors)) = future::block_on(future::poll_once(&mut task.0)) else {
        return false.into();
    };

    info!("Network connection established.");

    commands.remove_resource::<NetworkStartup>();
    commands.insert_resource(Sender(sender));
    commands.insert_resource(Receiver(receiver));
    commands.insert_resource(Errors(errors));

    true.into()
}

fn send_packages(mut events: ResMut<Events<SendPackageEvent>>, sender: Res<Sender>) {
    for event in events.drain() {
        if let Err(err) = sender.try_send(event.0) {
            match err {
                TrySendError::Full(_) => {
                    error!("Network stack is not keeping up. Skipping a message.");
                }
                TrySendError::Closed(_) => panic!("Network output channel is unexpectedly closed."),
            }
        }
    }
}

fn recv_packages(receiver: Res<Receiver>, mut events: EventWriter<PackageReceivedEvent>) {
    for _ in 0..MAX_RECV_PER_UPDATE {
        match receiver.try_recv() {
            Ok(package) => events.send(PackageReceivedEvent(package)),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Closed) => {
                panic!("Network message receiver is unexpectedly closed.");
            }
        }
    }

    warn!("More than {MAX_RECV_PER_UPDATE} messages received since the last update.");
}

fn recv_errors(receiver: Res<Errors>, mut events: EventWriter<ConnErrorEvent>) {
    loop {
        match receiver.try_recv() {
            Ok(error) => events.send(ConnErrorEvent(error)),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Closed) => {
                panic!("Network connection errors receiver is unexpectedly closed.");
            }
        }
    }
}
