use bevy::prelude::*;
use de_core::{gconfig::GameConfig, objects::Local, state::AppState};
use de_messages::ToPlayers;
use de_multiplayer::{NetEntities, NetRecvHealthEvent, ToPlayersEvent};
use de_objects::Health;
use de_signs::UpdateBarValueEvent;
use de_spawner::{DespawnActiveLocalEvent, DespawnerSet};

pub(crate) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LocalUpdateHealthEvent>()
            .add_event::<UpdateHealthEvent>()
            .add_systems(
                Update,
                (
                    (
                        update_local_health
                            .run_if(on_event::<LocalUpdateHealthEvent>())
                            .before(update_health),
                        update_remote_health
                            .run_if(on_event::<NetRecvHealthEvent>())
                            .before(update_health),
                        update_health.run_if(on_event::<UpdateHealthEvent>()),
                    )
                        .in_set(HealthSet::Update),
                    find_dead
                        .after(HealthSet::Update)
                        .before(DespawnerSet::Despawn),
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum HealthSet {
    Update,
}

/// Send this event to change health as a result of actions of locally
/// simulated entity.
#[derive(Event)]
pub(crate) struct LocalUpdateHealthEvent {
    entity: Entity,
    delta: f32,
}

impl LocalUpdateHealthEvent {
    /// # Panics
    ///
    /// Panics if health delta is not finite.
    pub(crate) fn new(entity: Entity, delta: f32) -> Self {
        assert!(delta.is_finite());
        Self { entity, delta }
    }
}

/// Send this event to change health of any entity.
#[derive(Event)]
struct UpdateHealthEvent {
    entity: Entity,
    delta: f32,
}

impl UpdateHealthEvent {
    /// # Panics
    ///
    /// Panics if health delta is not finite.
    fn new(entity: Entity, delta: f32) -> Self {
        assert!(delta.is_finite());
        Self { entity, delta }
    }
}

fn update_local_health(
    config: Res<GameConfig>,
    net_entities: NetEntities,
    mut in_events: EventReader<LocalUpdateHealthEvent>,
    mut out_events: EventWriter<UpdateHealthEvent>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    for event in in_events.iter() {
        out_events.send(UpdateHealthEvent::new(event.entity, event.delta));

        if config.multiplayer() {
            net_events.send(ToPlayersEvent::new(ToPlayers::ChangeHealth {
                entity: net_entities.net_id(event.entity),
                delta: event.delta.try_into().unwrap(),
            }));
        }
    }
}

fn update_remote_health(
    mut in_events: EventReader<NetRecvHealthEvent>,
    mut out_events: EventWriter<UpdateHealthEvent>,
) {
    for event in in_events.iter() {
        out_events.send(UpdateHealthEvent::new(event.entity(), event.delta()));
    }
}

fn update_health(
    mut healths: Query<&mut Health>,
    mut health_events: EventReader<UpdateHealthEvent>,
    mut bar_events: EventWriter<UpdateBarValueEvent>,
) {
    for event in health_events.iter() {
        let Ok(mut health) = healths.get_mut(event.entity) else {
            continue;
        };
        health.update(event.delta);
        bar_events.send(UpdateBarValueEvent::new(event.entity, health.fraction()));
    }
}

type LocallyChangedHealth<'w, 's> =
    Query<'w, 's, (Entity, &'static Health), (With<Local>, Changed<Health>)>;

fn find_dead(
    entities: LocallyChangedHealth,
    mut event_writer: EventWriter<DespawnActiveLocalEvent>,
) {
    for (entity, health) in entities.iter() {
        if health.destroyed() {
            event_writer.send(DespawnActiveLocalEvent::new(entity));
        }
    }
}
