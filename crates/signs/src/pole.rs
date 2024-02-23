use ahash::{AHashMap, AHashSet};
use bevy::prelude::*;
use de_core::{
    cleanup::DespawnOnGameExit, objects::ObjectTypeComponent, state::AppState, vecord::Vec2Ord,
};
use de_objects::{AssetCollection, SceneType, Scenes};
use de_types::projection::ToAltitude;

pub(crate) struct PolePlugin;

impl Plugin for PolePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdatePoleLocationEvent>()
            .add_event::<UpdatePoleVisibilityEvent>()
            .add_event::<SpawnPoleEvent>()
            .add_event::<DespawnPoleEvent>()
            .add_event::<MovePoleEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                PostUpdate,
                (
                    location_events.in_set(PolesSet::LocationEvents),
                    visibility_events.in_set(PolesSet::VisibilityEvents),
                    despawned
                        .in_set(PolesSet::Despawned)
                        .after(PolesSet::LocationEvents),
                    update_poles
                        .run_if(resource_exists_and_changed::<OwnersToPoles>())
                        .after(PolesSet::LocationEvents)
                        .after(PolesSet::VisibilityEvents)
                        .after(PolesSet::Despawned)
                        .before(PolesSet::SpawnPoles)
                        .before(PolesSet::DespawnPoles)
                        .before(PolesSet::MovePoles),
                    spawn_poles
                        .run_if(on_event::<SpawnPoleEvent>())
                        .in_set(PolesSet::SpawnPoles)
                        .after(PolesSet::DespawnPoles),
                    despawn_poles.in_set(PolesSet::DespawnPoles),
                    move_poles.before(PolesSet::DespawnPoles),
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum PolesSet {
    Despawned,
    VisibilityEvents,
    LocationEvents,
    SpawnPoles,
    DespawnPoles,
    MovePoles,
}

/// Send this event to add a new or update the position of a "target location"
/// pole to the map.
///
/// The location is remembered but the pole is not initially visible. Use
/// [`UpdatePoleVisibilityEvent`] to change visibility.
///
/// Each pole is associated with an entity and each entity may have up to one
/// associated pole. Note that this association is not managed via Bevy entity
/// hierarchy, thus visibility or transformation is independent.
#[derive(Event)]
pub struct UpdatePoleLocationEvent {
    owner: Entity,
    location: Vec2,
}

impl UpdatePoleLocationEvent {
    pub fn new(owner: Entity, location: Vec2) -> Self {
        Self { owner, location }
    }
}

/// Send this event to change visibility of an existing pole.
#[derive(Event)]
pub struct UpdatePoleVisibilityEvent {
    owner: Entity,
    visible: bool,
}

impl UpdatePoleVisibilityEvent {
    pub fn new(owner: Entity, visible: bool) -> Self {
        Self { owner, visible }
    }
}

/// Associations between entities "owning" poles and the pole properties.
#[derive(Default, Resource)]
struct OwnersToPoles(AHashMap<Entity, Pole>);

struct Pole {
    location: Vec2,
    visible: bool,
}

/// Spawn a new pole at this location. There must not be a pole already at this
/// location.
#[derive(Event)]
struct SpawnPoleEvent(Vec2);

/// Despawn an existing pole at this location.
#[derive(Event)]
struct DespawnPoleEvent(Vec2);

/// Move an existing pole from one location to another.
#[derive(Event)]
struct MovePoleEvent(Vec2, Vec2);

#[derive(Default, Resource)]
struct Poles(AHashMap<Vec2Ord, Entity>);

fn setup(mut commands: Commands) {
    commands.init_resource::<OwnersToPoles>();
    commands.init_resource::<Poles>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<OwnersToPoles>();
    commands.remove_resource::<Poles>();
}

fn location_events(
    mut owner_to_pole: ResMut<OwnersToPoles>,
    mut events: EventReader<UpdatePoleLocationEvent>,
) {
    for event in events.read() {
        owner_to_pole
            .0
            .entry(event.owner)
            .and_modify(|f| f.location = event.location)
            .or_insert_with(|| Pole {
                location: event.location,
                visible: false,
            });
    }
}

fn visibility_events(
    mut owner_to_pole: ResMut<OwnersToPoles>,
    mut events: EventReader<UpdatePoleVisibilityEvent>,
) {
    for event in events.read() {
        if let Some(pole) = owner_to_pole.0.get_mut(&event.owner) {
            pole.visible = event.visible;
        }
    }
}

fn despawned(
    mut owner_to_pole: ResMut<OwnersToPoles>,
    mut despawned: RemovedComponents<ObjectTypeComponent>,
) {
    for entity in despawned.read() {
        owner_to_pole.0.remove(&entity);
    }
}

fn update_poles(
    owner_to_pole: Res<OwnersToPoles>,
    poles: Res<Poles>,
    mut spawn_events: EventWriter<SpawnPoleEvent>,
    mut despawn_events: EventWriter<DespawnPoleEvent>,
    mut move_events: EventWriter<MovePoleEvent>,
) {
    let mut desired = AHashSet::with_capacity(owner_to_pole.0.len());
    for pole in owner_to_pole.0.values() {
        if pole.visible {
            desired.insert(Vec2Ord::from(pole.location));
        }
    }

    let mut to_despawn = Vec::new();
    for location in poles.0.keys() {
        if !desired.contains(location) {
            to_despawn.push(location.0);
        }
    }
    for location in &desired {
        if !poles.0.contains_key(location) {
            match to_despawn.pop() {
                Some(old) => move_events.send(MovePoleEvent(old, location.0)),
                None => spawn_events.send(SpawnPoleEvent(location.0)),
            }
        }
    }
    for location in to_despawn.drain(..) {
        despawn_events.send(DespawnPoleEvent(location));
    }
}

fn spawn_poles(
    mut commands: Commands,
    scenes: Res<Scenes>,
    mut poles: ResMut<Poles>,
    mut events: EventReader<SpawnPoleEvent>,
) {
    let scene = scenes.get(SceneType::Pole);
    for event in events.read() {
        let location = event.0.into();

        let transform = Transform::from_translation(event.0.to_msl());
        let entity = commands
            .spawn((
                transform,
                GlobalTransform::from(transform),
                VisibilityBundle::default(),
                DespawnOnGameExit,
                scene.clone(),
            ))
            .id();
        let result = poles.0.insert(location, entity);
        debug_assert!(result.is_none());
    }
}

fn despawn_poles(
    mut commands: Commands,
    mut poles: ResMut<Poles>,
    mut events: EventReader<DespawnPoleEvent>,
) {
    for event in events.read() {
        let entity = poles.0.remove(&event.0.into()).unwrap();
        commands.entity(entity).despawn_recursive();
    }
}

fn move_poles(
    mut poles: ResMut<Poles>,
    mut query: Query<&mut Transform>,
    mut events: EventReader<MovePoleEvent>,
) {
    for event in events.read() {
        let old = Vec2Ord::from(event.0);
        let new = Vec2Ord::from(event.1);
        let entity = poles.0.remove(&old).unwrap();
        query.get_mut(entity).unwrap().translation = event.1.to_msl();
        poles.0.insert(new, entity);
    }
}
