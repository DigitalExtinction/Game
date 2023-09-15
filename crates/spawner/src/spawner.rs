#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::{
    cleanup::DespawnOnGameExit,
    gconfig::GameConfig,
    objects::{Active, MovableSolid, ObjectTypeComponent, Playable, StaticSolid},
    player::PlayerComponent,
    state::AppState,
};
use de_energy::Battery;
use de_objects::{AssetCollection, InitialHealths, SceneType, Scenes, SolidObjects};
use de_pathing::{PathTarget, UpdateEntityPathEvent};
use de_terrain::{CircleMarker, MarkerVisibility, RectangleMarker};
use de_types::{
    objects::{ActiveObjectType, InactiveObjectType, ObjectType},
    player::Player,
};

use crate::ObjectCounter;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnLocalActiveEvent>()
            .add_event::<SpawnActiveEvent>()
            .add_event::<SpawnInactiveEvent>()
            .add_event::<SpawnEvent>()
            .add_systems(
                Update,
                (
                    spawn_local_active.before(spawn_active),
                    spawn_active.before(spawn),
                    spawn_inactive.before(spawn),
                    spawn,
                )
                    .in_set(SpawnerSet::Spawner)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum SpawnerSet {
    Spawner,
}

/// Send this event to spawn a new locally simulated active object.
#[derive(Event)]
pub struct SpawnLocalActiveEvent {
    object_type: ActiveObjectType,
    transform: Transform,
    player: Player,
    path_target: Option<PathTarget>,
}

impl SpawnLocalActiveEvent {
    pub fn stationary(object_type: ActiveObjectType, transform: Transform, player: Player) -> Self {
        Self::new(object_type, transform, player, None)
    }

    pub fn new(
        object_type: ActiveObjectType,
        transform: Transform,
        player: Player,
        path_target: Option<PathTarget>,
    ) -> Self {
        Self {
            object_type,
            transform,
            player,
            path_target,
        }
    }
}

#[derive(Event)]
struct SpawnActiveEvent {
    entity: Entity,
    object_type: ActiveObjectType,
    transform: Transform,
    player: Player,
}

impl SpawnActiveEvent {
    fn new(
        entity: Entity,
        object_type: ActiveObjectType,
        transform: Transform,
        player: Player,
    ) -> Self {
        Self {
            entity,
            object_type,
            transform,
            player,
        }
    }
}

/// Send this event to spawn an inactive object.
#[derive(Event)]
pub struct SpawnInactiveEvent {
    object_type: InactiveObjectType,
    transform: Transform,
}

impl SpawnInactiveEvent {
    pub fn new(object_type: InactiveObjectType, transform: Transform) -> Self {
        Self {
            object_type,
            transform,
        }
    }
}

#[derive(Event)]
struct SpawnEvent {
    entity: Entity,
    object_type: ObjectType,
    transform: Transform,
}

impl SpawnEvent {
    fn new(entity: Entity, object_type: ObjectType, transform: Transform) -> Self {
        Self {
            entity,
            object_type,
            transform,
        }
    }
}

fn spawn_local_active(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut event_reader: EventReader<SpawnLocalActiveEvent>,
    mut event_writer: EventWriter<SpawnActiveEvent>,
    mut path_events: EventWriter<UpdateEntityPathEvent>,
) {
    for event in event_reader.iter() {
        let mut entity_commands = commands.spawn_empty();

        if config.locals().is_playable(event.player) || cfg!(feature = "godmode") {
            entity_commands.insert(Playable);
        }

        let entity = entity_commands.id();
        event_writer.send(SpawnActiveEvent::new(
            entity,
            event.object_type,
            event.transform,
            event.player,
        ));

        if let Some(path_target) = event.path_target {
            path_events.send(UpdateEntityPathEvent::new(entity, path_target));
        }
    }
}

fn spawn_active(
    mut commands: Commands,
    mut counter: ResMut<ObjectCounter>,
    solids: SolidObjects,
    healths: Res<InitialHealths>,
    mut event_reader: EventReader<SpawnActiveEvent>,
    mut event_writer: EventWriter<SpawnEvent>,
    mut audio_events: EventWriter<PlaySpatialAudioEvent>,
) {
    for event in event_reader.iter() {
        counter
            .player_mut(event.player)
            .update(event.object_type, 1);

        let mut entity_commands = commands.entity(event.entity);
        entity_commands.insert((
            Active,
            PlayerComponent::from(event.player),
            Battery::default(),
            MarkerVisibility::default(),
            healths.health(event.object_type).clone(),
        ));

        let solid = solids.get(ObjectType::Active(event.object_type));
        match event.object_type {
            ActiveObjectType::Building(_) => {
                let local_aabb = solid.ichnography().local_aabb();
                entity_commands.insert((
                    StaticSolid,
                    RectangleMarker::from_aabb_transform(local_aabb, &event.transform),
                ));

                audio_events.send(PlaySpatialAudioEvent::new(
                    Sound::Construct,
                    event.transform.translation,
                ));
            }
            ActiveObjectType::Unit(_) => {
                let radius = solid.ichnography().radius();
                entity_commands.insert((MovableSolid, CircleMarker::new(radius)));

                audio_events.send(PlaySpatialAudioEvent::new(
                    Sound::Manufacture,
                    event.transform.translation,
                ));
            }
        }

        if let Some(cannon) = solid.cannon() {
            entity_commands.insert(cannon.clone());
        }

        event_writer.send(SpawnEvent::new(
            entity_commands.id(),
            ObjectType::Active(event.object_type),
            event.transform,
        ));
    }
}

fn spawn_inactive(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnInactiveEvent>,
    mut event_writer: EventWriter<SpawnEvent>,
) {
    for event in event_reader.iter() {
        let entity = commands.spawn(StaticSolid).id();
        event_writer.send(SpawnEvent::new(
            entity,
            ObjectType::Inactive(event.object_type),
            event.transform,
        ));
    }
}

fn spawn(mut commands: Commands, scenes: Res<Scenes>, mut events: EventReader<SpawnEvent>) {
    for event in events.iter() {
        info!("Spawning object {}", event.object_type);
        let mut entity_commands = commands.entity(event.entity);

        entity_commands.insert((
            event.transform,
            GlobalTransform::from(event.transform),
            Visibility::Inherited,
            ComputedVisibility::default(),
            ObjectTypeComponent::from(event.object_type),
            scenes.get(SceneType::Solid(event.object_type)).clone(),
            DespawnOnGameExit,
        ));
    }
}
