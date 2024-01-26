use bevy::prelude::*;
use de_core::{
    gconfig::is_multiplayer,
    schedule::{Movement, PreMovement},
    state::AppState,
};
use de_messages::ToPlayers;
use de_multiplayer::{GameNetSet, NetEntities, NetRecvSetPathEvent, ToPlayersEvent};

use crate::{
    pplugin::{PathFoundEvent, PathingSet},
    ScheduledPath,
};

pub struct SyncingPlugin;

impl Plugin for SyncingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreMovement,
            receive_paths
                .run_if(on_event::<NetRecvSetPathEvent>())
                .run_if(in_state(AppState::InGame))
                .after(GameNetSet::Messages),
        )
        .add_systems(
            Movement,
            send_new_paths
                .run_if(is_multiplayer)
                .run_if(in_state(AppState::InGame))
                .after(PathingSet::PathResults),
        );
    }
}

fn receive_paths(mut commands: Commands, mut events: EventReader<NetRecvSetPathEvent>) {
    for event in events.read() {
        let mut entity_commands = commands.entity(event.entity());

        match event.path() {
            Some(path) => {
                entity_commands.insert(ScheduledPath::new(path.clone()));
            }
            None => {
                entity_commands.remove::<ScheduledPath>();
            }
        }
    }
}

fn send_new_paths(
    net_entities: NetEntities,
    mut path_events: EventReader<PathFoundEvent>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    for event in path_events.read() {
        net_events.send(ToPlayersEvent::new(ToPlayers::SetPath {
            entity: net_entities.local_net_id(event.entity()),
            waypoints: event.path().map(|p| p.try_into().unwrap()),
        }));
    }
}
