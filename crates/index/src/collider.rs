use de_objects::ObjectCollider;
use parry3d::{
    bounding_volume::{BoundingVolume, AABB},
    math::Isometry,
    query::{Ray, RayCast},
};

/// Entity collider with cached entity-space and world-space AABBs for fast
/// query pre-filtering.
pub struct LocalCollider {
    object_collider: ObjectCollider,
    /// World-space position of the collider.
    position: Isometry<f32>,
    /// Collider-space AABB.
    local_aabb: AABB,
    /// World-space AABB. It is kept for fast geometric pre-filtering.
    world_aabb: AABB,
}

impl LocalCollider {
    /// Creates a new entity collider from entity shape and position.
    pub fn new(object_collider: ObjectCollider, position: Isometry<f32>) -> Self {
        let local_aabb = object_collider.compute_aabb();
        let world_aabb = local_aabb.transform_by(&position);

        Self {
            object_collider,
            position,
            local_aabb,
            world_aabb,
        }
    }

    pub(crate) fn world_aabb(&self) -> &AABB {
        &self.world_aabb
    }

    /// Updates position of cached world-space AABB of the collider.
    pub(crate) fn update_position(&mut self, position: Isometry<f32>) {
        self.world_aabb = self.local_aabb.transform_by(&position);
        self.position = position;
    }

    pub(crate) fn cast_ray(&self, ray: &Ray, max_toi: f32) -> Option<f32> {
        if self.world_aabb.intersects_local_ray(ray, max_toi) {
            self.object_collider.cast_ray(&self.position, ray, max_toi)
        } else {
            None
        }
    }

    pub(crate) fn intersects(&self, rhs: &Self) -> bool {
        if self.world_aabb.intersects(&rhs.world_aabb) {
            self.object_collider
                .intersects(&self.position, &rhs.object_collider, &rhs.position)
        } else {
            false
        }
    }
}
