use ahash::{AHashMap, AHashSet};
use bevy::prelude::*;
use de_core::{
    baseset::GameSet, cleanup::DespawnOnGameExit, objects::ObjectType, projection::ToAltitude,
    state::AppState, vecord::Vec2Ord,
};
use de_objects::{AssetCollection, SceneType, Scenes};

pub(crate) struct PolePlugin;

impl Plugin for PolePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdatePoleLocationEvent>()
            .add_event::<UpdatePoleVisibilityEvent>()
            .add_event::<SpawnPoleEvent>()
            .add_event::<DespawnPoleEvent>()
            .add_event::<MovePoleEvent>()
            .add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                location_events
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(PolesSet::LocationEvents),
            )
            .add_system(
                visibility_events
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(PolesSet::VisibilityEvents),
            )
            .add_system(
                despawned
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(PolesSet::Despawned)
                    .after(PolesSet::LocationEvents),
            )
            .add_system(
                update_poles
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(resource_exists_and_changed::<OwnersToPoles>())
                    .after(PolesSet::LocationEvents)
                    .after(PolesSet::VisibilityEvents)
                    .after(PolesSet::Despawned)
                    .before(PolesSet::SpawnPoles)
                    .before(PolesSet::DespawnPoles)
                    .before(PolesSet::MovePoles),
            )
            .add_system(
                spawn_poles
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<SpawnPoleEvent>())
                    .in_set(PolesSet::SpawnPoles)
                    .after(PolesSet::DespawnPoles),
            )
            .add_system(
                despawn_poles
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(PolesSet::DespawnPoles),
            )
            .add_system(
                move_poles
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .before(PolesSet::DespawnPoles),
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
struct SpawnPoleEvent(Vec2);

/// Despawn an existing pole at this location.
struct DespawnPoleEvent(Vec2);

/// Move an existing pole from one location to another.
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
    for event in events.iter() {
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
    for event in events.iter() {
        if let Some(pole) = owner_to_pole.0.get_mut(&event.owner) {
            pole.visible = event.visible;
        }
    }
}

fn despawned(
    mut owner_to_pole: ResMut<OwnersToPoles>,
    mut despawned: RemovedComponents<ObjectType>,
) {
    for entity in despawned.iter() {
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
    for event in events.iter() {
        let location = event.0.into();

        let transform = Transform::from_translation(event.0.to_msl());
        let entity = commands
            .spawn((
                transform,
                GlobalTransform::from(transform),
                Visibility::Visible,
                ComputedVisibility::default(),
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
    for event in events.iter() {
        let entity = poles.0.remove(&event.0.into()).unwrap();
        commands.entity(entity).despawn_recursive();
    }
}

fn move_poles(
    mut poles: ResMut<Poles>,
    mut query: Query<&mut Transform>,
    mut events: EventReader<MovePoleEvent>,
) {
    for event in events.iter() {
        let old = Vec2Ord::from(event.0);
        let new = Vec2Ord::from(event.1);
        let entity = poles.0.remove(&old).unwrap();
        query.get_mut(entity).unwrap().translation = event.1.to_msl();
        poles.0.insert(new, entity);
    }
}
