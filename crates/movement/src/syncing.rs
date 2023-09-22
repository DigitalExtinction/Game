use std::time::Duration;

use bevy::prelude::*;
use de_core::{
    gamestate::GameState,
    gconfig::is_multiplayer,
    objects::{Local, MovableSolid},
    schedule::{Movement, PreMovement},
    state::AppState,
};
use de_messages::ToPlayers;
use de_multiplayer::{NetEntities, NetRecvTransformEvent, ToPlayersEvent};

use crate::movement::MovementSet;

const MIN_SYNC_PERIOD: Duration = Duration::from_secs(2);
const SYNC_RANDOMIZATION_MS: u64 = 2_000;

pub(crate) struct SyncingPlugin;

impl Plugin for SyncingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreMovement,
            setup_entities
                .run_if(is_multiplayer)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Movement,
            (
                receive_transforms
                    .run_if(on_event::<NetRecvTransformEvent>())
                    .after(MovementSet::UpdateTransform),
                send_transforms
                    .run_if(is_multiplayer)
                    .after(MovementSet::UpdateTransform),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
struct SyncTimer(Duration);

impl SyncTimer {
    fn schedule(time: Duration) -> Duration {
        let jitter = Duration::from_millis(fastrand::u64(0..SYNC_RANDOMIZATION_MS));
        time + MIN_SYNC_PERIOD + jitter
    }

    fn new(time: Duration) -> Self {
        Self(Self::schedule(time))
    }

    /// Sets sync expiration to the future relative to the current time.
    fn refresh(&mut self, time: Duration) {
        self.0 = Self::schedule(time);
    }

    /// Returns true if transform sync is already due.
    fn outdated(&self, time: Duration) -> bool {
        time >= self.0
    }
}

type NotSetUp = (With<MovableSolid>, With<Local>, Without<SyncTimer>);

fn setup_entities(mut commands: Commands, time: Res<Time>, entities: Query<Entity, NotSetUp>) {
    let time = time.elapsed();
    for entity in entities.iter() {
        commands.entity(entity).insert(SyncTimer::new(time));
    }
}

fn receive_transforms(
    mut entities: Query<&mut Transform>,
    mut events: EventReader<NetRecvTransformEvent>,
) {
    for event in events.iter() {
        if let Ok(mut transform) = entities.get_mut(event.entity()) {
            *transform = event.transform();
        }
    }
}

fn send_transforms(
    time: Res<Time>,
    net_entities: NetEntities,
    mut entities: Query<(Entity, &mut SyncTimer, &Transform)>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    let time = time.elapsed();
    for (entity, mut sync, transform) in entities.iter_mut() {
        if sync.outdated(time) {
            sync.refresh(time);

            net_events.send(ToPlayersEvent::new(ToPlayers::Transform {
                entity: net_entities.local_net_id(entity),
                transform: transform.into(),
            }));
        }
    }
}
