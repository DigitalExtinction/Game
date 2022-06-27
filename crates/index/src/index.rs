//! This module contains implementation of spatial index of entities and
//! various system parameters to retrieve entities based on spatial queries.

use std::cmp::Ordering;

use ahash::AHashMap;
use bevy::{
    ecs::{
        query::{Fetch, FilterFetch, WorldQuery},
        system::SystemParam,
    },
    prelude::{Entity, Query, Res},
};
use parry3d::{
    bounding_volume::{BoundingVolume, AABB},
    math::{Isometry, Point},
    query::Ray,
    shape::Segment,
};

use super::{collider::LocalCollider, grid::TileGrid, segment::SegmentCandidates};
use crate::{aabb::AabbCandidates, collider::ColliderWithCache};

/// 2D rectangular grid based spatial index of entities.
pub struct EntityIndex {
    grid: TileGrid,
    world_bounds: AABB,
    colliders: AHashMap<Entity, LocalCollider>,
}

impl EntityIndex {
    /// Creates a new empty index.
    // Needs to be public because it is used in a benchmark.
    pub fn new() -> Self {
        Self {
            grid: TileGrid::new(),
            world_bounds: AABB::new(Point::origin(), Point::origin()),
            colliders: AHashMap::new(),
        }
    }

    // Needs to be public because it is used in a benchmark.
    pub fn insert(&mut self, entity: Entity, collider: LocalCollider) {
        self.grid.insert(entity, collider.world_aabb());
        self.world_bounds.merge(collider.world_aabb());
        self.colliders.insert(entity, collider);
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        let collider = self
            .colliders
            .remove(&entity)
            .expect("Tried to remove non-existent entity.");
        self.grid.remove(entity, collider.world_aabb());
    }

    pub(crate) fn update(&mut self, entity: Entity, position: Isometry<f32>) {
        let collider = self
            .colliders
            .get_mut(&entity)
            .expect("Tried to update non-existent entity.");

        let old_aabb = *collider.world_aabb();
        collider.update_position(position);
        let new_aabb = collider.world_aabb();

        self.world_bounds.merge(new_aabb);
        self.grid.update(entity, &old_aabb, new_aabb);
    }

    /// Returns an iterator of potentially intersecting entities.
    fn cast_ray<'a>(&'a self, ray: &Ray, max_toi: f32) -> Option<SegmentCandidates<'a>> {
        let segment = match self.world_bounds.clip_ray_parameters(ray) {
            Some((param_start, param_stop)) => {
                debug_assert!(param_start <= param_stop);
                if param_start > max_toi {
                    return None;
                }
                let start = ray.origin + param_start * ray.dir;
                let stop = ray.origin + param_stop.min(max_toi) * ray.dir;
                Segment::new(start, stop)
            }
            None => return None,
        };
        Some(SegmentCandidates::new(&self.grid, segment))
    }

    /// Returns an iterator of potentially intersecting entities.
    fn query_aabb<'a>(&'a self, aabb: &AABB) -> AabbCandidates<'a> {
        AabbCandidates::new(&self.grid, aabb)
    }

    fn get_collider(&self, entity: Entity) -> &LocalCollider {
        self.colliders
            .get(&entity)
            .expect("Tried to get shape of a non-existent entity.")
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// System parameter implementing various spatial queries.
///
/// Only entities automatically indexed by systems from
/// [`super::systems::IndexPlugin`] could be retrieved.
#[derive(SystemParam)]
pub struct SpatialQuery<'w, 's, Q, F = ()>
where
    Q: WorldQuery + Sync + Send + 'static,
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    index: Res<'w, EntityIndex>,
    entities: Query<'w, 's, Q, F>,
}

impl<'w, 's, Q, F> SpatialQuery<'w, 's, Q, F>
where
    Q: WorldQuery + Sync + Send + 'static,
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    /// Returns closest entity whose shape, as indexed by systems registered by
    /// [`super::systems::IndexPlugin`], intersects a given ray.
    pub fn cast_ray(
        &self,
        ray: &Ray,
        max_toi: f32,
    ) -> Option<RayEntityIntersection<<<Q as WorldQuery>::ReadOnlyFetch as Fetch<'_, '_>>::Item>>
    {
        let candidate_sets = match self.index.cast_ray(ray, max_toi) {
            Some(candidates) => candidates,
            None => return None,
        };

        for candidates in candidate_sets {
            if let Some(intersection) = candidates
                .iter()
                .filter_map(|&candidate| match self.entities.get(candidate) {
                    Ok(item) => self
                        .index
                        .get_collider(candidate)
                        .cast_ray(ray, max_toi)
                        .map(|toi| RayEntityIntersection::new(candidate, toi, item)),
                    Err(_) => None,
                })
                .min()
            {
                // The sets are retrieved in order given by distance from ray
                // origin, thus the entity returned here is guaranteed to be
                // closer than any entity from sets visited later.
                return Some(intersection);
            }
        }

        None
    }

    /// Returns true if queried solid object on the map, as indexed by
    /// [`super::systems::IndexPlugin`], intersects with the given collider.
    pub fn collides(&self, collider: &impl ColliderWithCache) -> bool {
        let candidate_sets = self.index.query_aabb(collider.world_aabb());
        candidate_sets.flatten().any(|candidate| {
            self.entities.get(candidate).map_or(false, |_| {
                self.index.get_collider(candidate).intersects(collider)
            })
        })
    }
}

pub struct RayEntityIntersection<T> {
    entity: Entity,
    toi: f32,
    item: T,
}

impl<T> RayEntityIntersection<T> {
    fn new(entity: Entity, toi: f32, item: T) -> Self {
        Self { entity, toi, item }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Intersection time of impact. Intersection point is equal to
    /// `ray.origin + intersection.toi() * ray.dir`.
    pub fn toi(&self) -> f32 {
        self.toi
    }

    /// Single item (ECS world query result) associated with the intersected
    /// entity.
    pub fn item(&self) -> &T {
        &self.item
    }
}

impl<T> PartialOrd for RayEntityIntersection<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for RayEntityIntersection<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.toi < other.toi {
            Ordering::Less
        } else if self.toi > other.toi {
            Ordering::Greater
        } else {
            self.entity.cmp(&other.entity)
        }
    }
}

impl<T> PartialEq for RayEntityIntersection<T> {
    fn eq(&self, other: &Self) -> bool {
        self.toi == other.toi && self.entity == other.entity
    }
}

impl<T> Eq for RayEntityIntersection<T> {}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use de_objects::ObjectCollider;
    use parry3d::{
        bounding_volume::AABB,
        math::{Isometry, Point, Vector},
        shape::{Cuboid, TriMesh, TriMeshFlags},
    };

    use super::*;

    #[test]
    fn test_entity_index() {
        let entity_a = Entity::from_raw(1);
        let mut trimesh_a: TriMesh = Cuboid::new(Vector::new(1., 2., 3.)).into();
        trimesh_a.set_flags(TriMeshFlags::ORIENTED).unwrap();
        let collider_a = LocalCollider::new(
            ObjectCollider::new(trimesh_a),
            Isometry::new(Vector::new(7., 0., 0.), Vector::new(0., 0., 0.)),
        );
        let entity_b = Entity::from_raw(2);
        let mut trimesh_b: TriMesh = Cuboid::new(Vector::new(2., 1., 2.)).into();
        trimesh_b.set_flags(TriMeshFlags::ORIENTED).unwrap();
        let collider_b = LocalCollider::new(
            ObjectCollider::new(trimesh_b),
            Isometry::new(Vector::new(7., 1000., 0.), Vector::new(0.1, 0., 0.)),
        );
        let position_b_2 = Isometry::new(Vector::new(7., 1000., -200.), Vector::new(0., 0., 0.));
        let entity_c = Entity::from_raw(3);
        let mut trimesh_c: TriMesh = Cuboid::new(Vector::new(2., 1., 2.)).into();
        trimesh_c.set_flags(TriMeshFlags::ORIENTED).unwrap();
        let collider_c = LocalCollider::new(
            ObjectCollider::new(trimesh_c),
            Isometry::new(Vector::new(7., 1000., 1000.), Vector::new(0.1, 0., 0.)),
        );

        let ray_a = Ray::new(Point::new(0., 0.1, 0.), Vector::new(1., 0., 0.));
        let ray_b = Ray::new(Point::new(-10., 0.1, 0.), Vector::new(-1., 0., 0.));

        let mut index = EntityIndex::new();
        assert!(index.cast_ray(&ray_a, 120.).is_none());

        index.insert(entity_a, collider_a);
        index.insert(entity_b, collider_b);
        index.insert(entity_c, collider_c);

        assert_eq!(
            index.get_collider(entity_a).world_aabb(),
            &AABB::new(Point::new(6., -2., -3.), Point::new(8., 2., 3.))
        );
        let entities_a: AHashSet<Entity> =
            index.cast_ray(&ray_a, 120.).unwrap().flatten().collect();
        assert_eq!(entities_a, AHashSet::from_iter(vec![entity_a, entity_b]));

        index.update(entity_b, position_b_2);
        assert_eq!(
            index.get_collider(entity_b).world_aabb(),
            &AABB::new(Point::new(5., 999., -202.), Point::new(9., 1001., -198.))
        );
        let entities_b: AHashSet<Entity> =
            index.cast_ray(&ray_a, 120.).unwrap().flatten().collect();
        assert_eq!(entities_b, AHashSet::from_iter(vec![entity_a]));

        index.remove(entity_a);
        let entities_c: AHashSet<Entity> =
            index.cast_ray(&ray_a, 120.).unwrap().flatten().collect();
        assert_eq!(entities_c, AHashSet::new());

        assert!(index.cast_ray(&ray_b, 120.).is_none());
    }

    #[test]
    fn test_entity_collider() {
        let mut trimesh: TriMesh = Cuboid::new(Vector::new(1., 2., 3.)).into();
        trimesh.set_flags(TriMeshFlags::ORIENTED).unwrap();
        let object_collider = ObjectCollider::new(trimesh);
        let position_a = Isometry::new(Vector::new(7., 0., 0.), Vector::new(0., 0., 0.));
        let position_b = Isometry::new(Vector::new(9., 0., 0.), Vector::new(0., 0., 0.));
        let ray = Ray::new(Point::new(0., 0., 0.), Vector::new(1., 0., 0.));

        let mut collider = LocalCollider::new(object_collider, position_a);
        assert_eq!(
            collider.world_aabb(),
            &AABB::new(Point::new(6., -2., -3.), Point::new(8., 2., 3.))
        );

        let intersection_a = collider.cast_ray(&ray, f32::INFINITY).unwrap();
        assert_eq!(intersection_a, 6.);

        collider.update_position(position_b);
        assert_eq!(
            collider.world_aabb(),
            &AABB::new(Point::new(8., -2., -3.), Point::new(10., 2., 3.))
        );

        let intersection_b = collider.cast_ray(&ray, f32::INFINITY).unwrap();
        assert_eq!(intersection_b, 8.);
    }

    #[test]
    fn test_ray_entity_intersection() {
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let entity_c = Entity::from_raw(3);

        let intersection_a = RayEntityIntersection::new(entity_a, 0.5, ());
        let intersection_b = RayEntityIntersection::new(entity_b, 0.5, ());
        let intersection_c = RayEntityIntersection::new(entity_c, 1.5, ());

        assert_eq!(intersection_a.cmp(&intersection_a), Ordering::Equal);
        assert_eq!(intersection_a.cmp(&intersection_b), Ordering::Less);
        assert_eq!(intersection_b.cmp(&intersection_a), Ordering::Greater);
        assert_eq!(intersection_a.cmp(&intersection_c), Ordering::Less);
        assert_eq!(intersection_c.cmp(&intersection_a), Ordering::Greater);
    }
}
