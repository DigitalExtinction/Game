use bevy::prelude::*;
use de_core::{baseset::GameSet, gamestate::GameState, projection::ToFlat};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPathEvent};

pub(crate) struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChaseTargetEvent>()
            .add_system(
                handle_chase_events
                    .in_base_set(GameSet::PreUpdate)
                    .run_if(in_state(GameState::Playing))
                    .in_set(ChaseSet::ChaseTargetEvent),
            )
            .add_system(
                chase
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub enum ChaseSet {
    ChaseTargetEvent,
}

/// Send this event to start or stop chasing of an entity (movable or static).
pub struct ChaseTargetEvent {
    entity: Entity,
    target: Option<ChaseTarget>,
}

impl ChaseTargetEvent {
    /// Creates a new chase event.
    ///
    /// # Arguments
    ///
    /// * `entity` - the chasing entity.
    ///
    /// * `target` - target to chase or None if chasing shall be stopped.
    pub fn new(entity: Entity, target: Option<ChaseTarget>) -> Self {
        Self { entity, target }
    }

    fn entity(&self) -> Entity {
        self.entity
    }

    fn target(&self) -> Option<&ChaseTarget> {
        self.target.as_ref()
    }
}

/// Units with this component will chase the target entity.
#[derive(Component, Deref)]
struct ChaseTargetComponent(ChaseTarget);

impl ChaseTargetComponent {
    fn new(target: ChaseTarget) -> Self {
        Self(target)
    }
}

#[derive(Clone)]
pub struct ChaseTarget {
    target: Entity,
    min_distance: f32,
    max_distance: f32,
}

impl ChaseTarget {
    /// Creates a new chase target.
    ///
    /// # Arguments
    ///
    /// * `target` - entity to chase.
    ///
    /// * `min_distance` - minimum distance between the chasing entity and the
    ///   cased entity. Elevation is ignored during the distance calculation.
    ///
    /// * `max_distance` - maximum distance between the chasing entity and the
    ///   cased entity. Elevation is ignored during the distance calculation.
    ///
    /// # Panics
    ///
    /// May panic if `min_distance` or `max_distance` is not non-negative
    /// finite number or when `min_distance` is greater or equal to
    /// `max_distance`.
    pub fn new(target: Entity, min_distance: f32, max_distance: f32) -> Self {
        debug_assert!(min_distance.is_finite());
        debug_assert!(max_distance.is_finite());
        debug_assert!(min_distance >= 0.);
        debug_assert!(max_distance >= 0.);
        debug_assert!(min_distance < max_distance);

        Self {
            target,
            min_distance,
            max_distance,
        }
    }

    pub fn target(&self) -> Entity {
        self.target
    }

    fn min_distance(&self) -> f32 {
        self.min_distance
    }

    fn max_distance(&self) -> f32 {
        self.max_distance
    }
}

fn handle_chase_events(mut commands: Commands, mut events: EventReader<ChaseTargetEvent>) {
    for event in events.iter() {
        let mut entity_commands = commands.entity(event.entity());
        match event.target() {
            Some(target) => entity_commands.insert(ChaseTargetComponent::new(target.clone())),
            None => entity_commands.remove::<ChaseTargetComponent>(),
        };
    }
}

fn chase(
    mut commands: Commands,
    mut path_events: EventWriter<UpdateEntityPathEvent>,
    chasing: Query<(
        Entity,
        &Transform,
        &ChaseTargetComponent,
        Option<&PathTarget>,
    )>,
    targets: Query<&Transform>,
) {
    for (entity, transform, chase_target, path_target) in chasing.iter() {
        let target_position = match targets.get(chase_target.target()) {
            Ok(transform) => transform.translation.to_flat(),
            Err(_) => {
                commands.entity(entity).remove::<ChaseTargetComponent>();
                continue;
            }
        };

        let (path_target, distance) = path_target
            .map(|path_target| (path_target.location(), path_target.properties().distance()))
            .unwrap_or((transform.translation.to_flat(), 0.));

        if (target_position - path_target).length() + distance <= chase_target.max_distance() {
            continue;
        }

        path_events.send(UpdateEntityPathEvent::new(
            entity,
            PathTarget::new(
                target_position,
                PathQueryProps::new(chase_target.min_distance(), chase_target.max_distance()),
                true,
            ),
        ));
    }
}
