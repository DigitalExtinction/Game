use std::ops::Deref;

use bevy::prelude::*;

use crate::{config::NetGameConf, NetState};

pub(super) struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartMultiplayerEvent>()
            .add_event::<ShutdownMultiplayerEvent>()
            .add_system(cleanup.in_schedule(OnEnter(NetState::None)))
            .add_system(
                start
                    .run_if(in_state(NetState::None))
                    .run_if(on_event::<StartMultiplayerEvent>()),
            )
            .add_system(
                shutdown
                    .run_if(not(in_state(NetState::None)))
                    .run_if(on_event::<ShutdownMultiplayerEvent>()),
            );
    }
}

/// Send this event to setup and startup multiplayer functionality.
///
/// These events are processed only in [`crate::netstate::NetState::None`]
/// state.
pub struct StartMultiplayerEvent {
    net_conf: NetGameConf,
}

impl StartMultiplayerEvent {
    pub fn new(net_conf: NetGameConf) -> Self {
        Self { net_conf }
    }
}

/// Send this event to shutdown multiplayer functionality.
pub struct ShutdownMultiplayerEvent;

#[derive(Resource)]
pub(crate) struct NetGameConfRes(NetGameConf);

impl Deref for NetGameConfRes {
    type Target = NetGameConf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<NetGameConfRes>();
}

fn start(
    mut commands: Commands,
    mut next_state: ResMut<NextState<NetState>>,
    mut events: ResMut<Events<StartMultiplayerEvent>>,
) {
    let Some(event) = events.drain().last() else {
        return;
    };

    commands.insert_resource(NetGameConfRes(event.net_conf));
    next_state.set(NetState::Connecting);
}

fn shutdown(mut next_state: ResMut<NextState<NetState>>) {
    next_state.set(NetState::None);
}
