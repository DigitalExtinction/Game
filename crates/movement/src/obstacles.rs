use bevy::prelude::*;
use de_core::{
    schedule::{Movement, PreMovement},
    gamestate::GameState,
    objects::{MovableSolid, ObjectType, StaticSolid},
    projection::ToFlat,
};
use de_index::SpatialQuery;
use de_objects::SolidObjects;
use parry3d::{bounding_volume::Aabb, math::Point};

use crate::{cache::DecayingCache, disc::Disc};

/// Obstacle avoidance algorithm takes into account only obstacles inside a
/// rectangle of this half-size.
const NEARBY_HALF_EXTENT: f32 = 10.;

pub(crate) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreMovement,
            (
                setup_discs.run_if(in_state(GameState::Playing)),
                update_discs.run_if(in_state(GameState::Playing)),
            ),
        )
        .add_systems(
            Movement,
            (
                update_nearby::<StaticObstacles, StaticSolid>
                    .run_if(in_state(GameState::Playing))
                    .in_set(ObstaclesLables::UpdateNearby),
                update_nearby::<MovableObstacles, MovableSolid>
                    .run_if(in_state(GameState::Playing))
                    .in_set(ObstaclesLables::UpdateNearby),
            ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum ObstaclesLables {
    UpdateNearby,
}

pub(crate) struct StaticObstacles;

pub(crate) struct MovableObstacles;

type Uninitialized<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static ObjectType),
    (With<MovableSolid>, Without<Disc>),
>;

fn setup_discs(mut commands: Commands, solids: SolidObjects, objects: Uninitialized) {
    for (entity, transform, &object_type) in objects.iter() {
        let center = transform.translation.to_flat();
        let radius = solids.get(object_type).ichnography().radius();
        commands.entity(entity).insert((
            Disc::new(center, radius),
            DecayingCache::<StaticObstacles>::default(),
            DecayingCache::<MovableObstacles>::default(),
        ));
    }
}

fn update_discs(mut objects: Query<(&Transform, &mut Disc), Changed<Transform>>) {
    for (transform, mut disc) in objects.iter_mut() {
        disc.set_center(transform.translation.to_flat());
    }
}

fn update_nearby<M: Send + Sync + 'static, T: Component>(
    time: Res<Time>,
    mut objects: Query<(Entity, &Transform, &mut DecayingCache<M>)>,
    space: SpatialQuery<Entity, With<T>>,
) {
    objects
        .par_iter_mut()
        .for_each_mut(|(entity, transform, mut cache)| {
            cache.clear();
            let half_extent = Vec3::splat(NEARBY_HALF_EXTENT);
            let mins = transform.translation - half_extent;
            let maxs = transform.translation + half_extent;
            let region = Aabb::new(Point::from(mins), Point::from(maxs));
            cache.extend(space.query_aabb(&region, Some(entity)));
            cache.decay(time.delta_seconds());
        });
}
