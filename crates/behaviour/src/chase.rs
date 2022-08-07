use bevy::prelude::*;
use de_core::{projection::ToFlat, state::GameState};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPath};
use iyes_loopless::prelude::*;

pub(crate) struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new().with_system(chase.run_in_state(GameState::Playing)),
        );
    }
}

/// Units with this component will chase the target entity.
#[derive(Component)]
pub struct ChaseTarget {
    entity: Entity,
    min_distance: f32,
    max_distance: f32,
}

impl ChaseTarget {
    /// Creates a new chase target.
    ///
    /// # Arguments
    ///
    /// * `entity` - entity to chase.
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
    pub fn new(entity: Entity, min_distance: f32, max_distance: f32) -> Self {
        debug_assert!(min_distance.is_finite());
        debug_assert!(max_distance.is_finite());
        debug_assert!(min_distance >= 0.);
        debug_assert!(max_distance >= 0.);
        debug_assert!(min_distance < max_distance);

        Self {
            entity,
            min_distance,
            max_distance,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    fn min_distance(&self) -> f32 {
        self.min_distance
    }

    fn max_distance(&self) -> f32 {
        self.max_distance
    }
}

fn chase(
    mut commands: Commands,
    mut path_events: EventWriter<UpdateEntityPath>,
    chasing: Query<(Entity, &GlobalTransform, &ChaseTarget, Option<&PathTarget>)>,
    targets: Query<&GlobalTransform>,
) {
    for (entity, transform, chase_target, path_target) in chasing.iter() {
        let target_position = match targets.get(chase_target.entity()) {
            Ok(transform) => transform.translation().to_flat(),
            Err(_) => {
                commands.entity(entity).remove::<ChaseTarget>();
                continue;
            }
        };

        let (path_target, distance) = path_target
            .map(|path_target| (path_target.location(), path_target.properties().distance()))
            .unwrap_or((transform.translation().to_flat(), 0.));

        if (target_position - path_target).length() + distance <= chase_target.max_distance() {
            continue;
        }

        path_events.send(UpdateEntityPath::new(
            entity,
            PathTarget::new(
                target_position,
                PathQueryProps::new(chase_target.min_distance(), chase_target.max_distance()),
                true,
            ),
        ));
    }
}
