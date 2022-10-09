use bevy::prelude::*;
use de_core::{
    objects::{MovableSolid, ObjectType, StaticSolid},
    projection::ToFlat,
    stages::GameStage,
    state::GameState,
};
use de_index::SpatialQuery;
use de_objects::{IchnographyCache, ObjectCache};
use iyes_loopless::prelude::*;
use parry2d::bounding_volume::BoundingSphere;
use parry3d::{bounding_volume::AABB, math::Point};

use crate::cache::DecayingCache;

/// Obstacle avoidance algorithm takes into account only obstacles inside a
/// rectangle of this half-size.
const NEARBY_HALF_EXTENT: f32 = 10.;

pub(crate) struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::PreMovement,
            SystemSet::new()
                .with_system(setup_discs.run_in_state(GameState::Playing))
                .with_system(update_discs.run_in_state(GameState::Playing)),
        )
        .add_system_set_to_stage(
            GameStage::Movement,
            SystemSet::new()
                .with_system(
                    update_nearby::<StaticObstacles, StaticSolid>
                        .run_in_state(GameState::Playing)
                        .label(ObstaclesLables::UpdateNearby),
                )
                .with_system(
                    update_nearby::<MovableObstacles, MovableSolid>
                        .run_in_state(GameState::Playing)
                        .label(ObstaclesLables::UpdateNearby),
                ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum ObstaclesLables {
    UpdateNearby,
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct Disc(BoundingSphere);

pub(crate) struct StaticObstacles;

pub(crate) struct MovableObstacles;

type Uninitialized<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static ObjectType),
    (With<MovableSolid>, Without<Disc>),
>;

fn setup_discs(mut commands: Commands, cache: Res<ObjectCache>, objects: Uninitialized) {
    for (entity, transform, &object_type) in objects.iter() {
        let center = transform.translation.to_flat().into();
        let footprint = cache.get_ichnography(object_type).convex_hull();
        let radius = footprint
            .points()
            .iter()
            .map(|p| p.coords.norm())
            .max_by(f32::total_cmp)
            .unwrap();
        commands
            .entity(entity)
            .insert(Disc(BoundingSphere::new(center, radius)))
            .insert(DecayingCache::<StaticObstacles>::default())
            .insert(DecayingCache::<MovableObstacles>::default());
    }
}

fn update_discs(mut objects: Query<(&Transform, &mut Disc), Changed<Transform>>) {
    for (transform, mut disc) in objects.iter_mut() {
        disc.center = transform.translation.to_flat().into();
    }
}

fn update_nearby<M: Send + Sync + 'static, T: Component>(
    time: Res<Time>,
    mut objects: Query<(Entity, &Transform, &mut DecayingCache<M>)>,
    space: SpatialQuery<Entity, With<T>>,
) {
    objects.par_for_each_mut(512, |(entity, transform, mut cache)| {
        cache.clear();
        let half_extent = Vec3::splat(NEARBY_HALF_EXTENT);
        let mins = transform.translation - half_extent;
        let maxs = transform.translation + half_extent;
        let region = AABB::new(Point::from(mins), Point::from(maxs));
        cache.extend(space.query_aabb(&region, Some(entity)));
        cache.decay(time.delta_seconds());
    });
}
