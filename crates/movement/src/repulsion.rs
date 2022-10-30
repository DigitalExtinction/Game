use bevy::prelude::*;
use de_core::{
    objects::{MovableSolid, ObjectType, StaticSolid},
    projection::ToFlat,
    stages::GameStage,
    state::GameState,
};
use de_map::size::MapBounds;
use de_objects::{IchnographyCache, ObjectCache, EXCLUSION_OFFSET};
use iyes_loopless::prelude::*;
use parry2d::{math::Isometry, na::Unit, query::PointQuery};

use crate::{
    cache::DecayingCache,
    disc::Disc,
    movement::DesiredMovement,
    obstacles::{MovableObstacles, ObstaclesLables, StaticObstacles},
    pathing::PathingLabels,
    MAX_ACCELERATION, MAX_SPEED,
};

const MAX_REPULSION_DISTANCE: f32 = 4.0;
const MIN_STATIC_OBJECT_DISTANCE: f32 = 1.;
const MIN_MOVABLE_OBJECT_DISTANCE: f32 = 0.5;
const REPULSION_FACTOR: f32 = 0.6;

pub(crate) struct RepulsionPlugin;

impl Plugin for RepulsionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::PreMovement,
            SystemSet::new().with_system(setup_entities.run_in_state(GameState::Playing)),
        )
        .add_system_set_to_stage(
            GameStage::Movement,
            SystemSet::new()
                .with_system(
                    repel_static
                        .run_in_state(GameState::Playing)
                        .label(RepulsionLables::RepelStatic)
                        .after(ObstaclesLables::UpdateNearby)
                        .after(PathingLabels::FollowPath),
                )
                .with_system(
                    repel_movable
                        .run_in_state(GameState::Playing)
                        .label(RepulsionLables::RepelMovable)
                        .after(ObstaclesLables::UpdateNearby)
                        .after(PathingLabels::FollowPath),
                )
                .with_system(
                    repel_bounds
                        .run_in_state(GameState::Playing)
                        .label(RepulsionLables::RepelBounds)
                        .after(PathingLabels::FollowPath),
                )
                .with_system(
                    apply
                        .run_in_state(GameState::Playing)
                        .label(RepulsionLables::Apply)
                        .after(RepulsionLables::RepelStatic)
                        .after(RepulsionLables::RepelMovable)
                        .after(RepulsionLables::RepelBounds),
                ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum RepulsionLables {
    RepelStatic,
    RepelMovable,
    RepelBounds,
    Apply,
}

/// This component collects directional bounds and computes bounded desired
/// velocity based on the bounds.
#[derive(Component, Default)]
struct Repulsion(Vec<DirectionBound>);

impl Repulsion {
    /// Adds another bound.
    ///
    /// # Arguments
    ///
    /// * `direction` - direction to the closest point of a particular
    ///   obstacle.
    ///
    /// * `room` - how much the repelled object can move along `direction`
    ///   until it reaches the point of minimum allowed distance. Note that
    ///   minimum allowed distance might be larger than zero.
    fn add(&mut self, direction: Vec2, room: f32) {
        let mut max = REPULSION_FACTOR * (2. * MAX_ACCELERATION).sqrt();
        if room > 0. {
            max *= room.sqrt();
        } else {
            max *= room;
        }
        self.0.push(DirectionBound::new(direction, max));
    }

    /// Computes a velocity constrained by all accumulated bounds.
    fn apply(&self, mut velocity: Vec2) -> Vec2 {
        for bound in &self.0 {
            bound.limit_max(&mut velocity)
        }
        // Since maximum speed along an axis might be smaller than zero, a
        // group of objects can push an object through another object. This
        // second loop prevents such a situation.
        for bound in &self.0 {
            bound.limit_zero(&mut velocity)
        }
        velocity
    }

    /// Clears all accumulated bounds.
    fn clear(&mut self) {
        self.0.clear()
    }
}

struct DirectionBound(Vec2, f32);

impl DirectionBound {
    fn new(dir: Vec2, max: f32) -> Self {
        Self(dir, max)
    }

    fn limit_max(&self, velocity: &mut Vec2) {
        self.limit(velocity, self.1)
    }

    fn limit_zero(&self, velocity: &mut Vec2) {
        self.limit(velocity, self.1.max(0.))
    }

    fn limit(&self, velocity: &mut Vec2, max: f32) {
        let projection = self.0.dot(*velocity);
        if projection > max {
            let correction = projection - max;
            *velocity -= correction * self.0;
        }
    }
}

fn setup_entities(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<Repulsion>)>,
) {
    for entity in objects.iter() {
        commands.entity(entity).insert(Repulsion::default());
    }
}

fn repel_static(
    cache: Res<ObjectCache>,
    mut objects: Query<(
        &DesiredMovement,
        &Disc,
        &DecayingCache<StaticObstacles>,
        &mut Repulsion,
    )>,
    obstacles: Query<(&ObjectType, &Transform), With<StaticSolid>>,
) {
    objects.par_for_each_mut(512, |(movement, disc, static_obstacles, mut repulsion)| {
        if movement.stopped() {
            return;
        }

        for &entity in static_obstacles.entities() {
            let (&object_type, transform) = obstacles.get(entity).unwrap();

            let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
            let isometry = Isometry::new(transform.translation.to_flat().into(), angle);
            let local_point = isometry.inverse_transform_point(&From::from(disc.center()));

            let footprint = cache.get_ichnography(object_type).convex_hull();
            let projection = footprint.project_local_point(&local_point, true);

            let mut diff = projection.point - local_point;
            let mut distance = diff.norm();
            if projection.is_inside {
                diff *= -1.;
                distance *= -1.;
            }
            distance -= disc.radius();

            if distance > MAX_REPULSION_DISTANCE {
                continue;
            }

            let direction = match Unit::try_new(diff, parry2d::math::DEFAULT_EPSILON) {
                Some(direction) => {
                    let feature_id = footprint.support_feature_id_toward(&direction);
                    let local_normal = footprint.feature_normal(feature_id).unwrap();
                    Vec2::from(isometry.transform_vector(&local_normal))
                }
                None => Vec2::X,
            };
            repulsion.add(direction, distance - MIN_STATIC_OBJECT_DISTANCE);
        }
    });
}

fn repel_movable(
    mut objects: Query<(
        &DesiredMovement,
        &Disc,
        &DecayingCache<MovableObstacles>,
        &mut Repulsion,
    )>,
    obstacles: Query<&Disc>,
) {
    objects.par_for_each_mut(512, |(movement, disc, movable_obstacles, mut repulsion)| {
        if movement.stopped() {
            return;
        }

        for &entity in movable_obstacles.entities() {
            let other_disc = obstacles.get(entity).unwrap();
            let diff = other_disc.center() - disc.center();
            let mut distance = diff.length();
            let direction = if distance <= parry2d::math::DEFAULT_EPSILON {
                Vec2::X
            } else {
                diff / distance
            };
            distance -= disc.radius() + other_disc.radius();
            if distance < MAX_REPULSION_DISTANCE {
                repulsion.add(direction, distance - MIN_MOVABLE_OBJECT_DISTANCE);
            }
        }
    });
}

fn repel_bounds(
    bounds: Res<MapBounds>,
    mut objects: Query<(&DesiredMovement, &Disc, &mut Repulsion)>,
) {
    objects.par_for_each_mut(512, |(movement, disc, mut repulsion)| {
        if movement.stopped() {
            return;
        }

        let projection = bounds
            .aabb()
            .project_local_point(&From::from(disc.center()), false);
        debug_assert!(projection.is_inside);

        let diff = Vec2::from(projection.point) - disc.center();
        let diff_norm = diff.length();
        let distance = diff_norm - disc.radius();

        if distance < MAX_REPULSION_DISTANCE {
            repulsion.add(diff / diff_norm, distance - EXCLUSION_OFFSET);
        }
    });
}

fn apply(mut objects: Query<(&mut Repulsion, &mut DesiredMovement)>) {
    objects.par_for_each_mut(512, |(mut repulsion, mut movement)| {
        let velocity = repulsion.apply(movement.velocity());
        movement.update(velocity.clamp_length_max(MAX_SPEED));
        repulsion.clear();
    });
}
