use de_objects::ObjectCollider;
use parry3d::{
    bounding_volume::{Aabb, BoundingVolume},
    math::Isometry,
    query::{Ray, RayCast},
};

pub trait ColliderWithCache {
    /// World-space AABB of the collider.
    fn world_aabb(&self) -> &Aabb;

    /// World-to-collider mapping.
    fn position(&self) -> &Isometry<f32>;

    fn inner(&self) -> &ObjectCollider;
}

/// Entity collider with cached entity-space and world-space AABBs for fast
/// query pre-filtering.
pub struct LocalCollider {
    object_collider: ObjectCollider,
    /// World-space position of the collider.
    position: Isometry<f32>,
    /// Collider-space AABB.
    local_aabb: Aabb,
    /// World-space AABB. It is kept for fast geometric pre-filtering.
    world_aabb: Aabb,
}

impl LocalCollider {
    /// Creates a new entity collider from entity shape and position.
    // Needs to be public because it is used in a benchmark.
    pub fn new(object_collider: ObjectCollider, position: Isometry<f32>) -> Self {
        let local_aabb = object_collider.aabb();
        let world_aabb = local_aabb.transform_by(&position);

        Self {
            object_collider,
            position,
            local_aabb,
            world_aabb,
        }
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

    pub(crate) fn intersects(&self, rhs: &impl ColliderWithCache) -> bool {
        if self.query_aabb(rhs.world_aabb()) {
            self.object_collider
                .intersects(&self.position, rhs.inner(), rhs.position())
        } else {
            false
        }
    }

    /// Returns true if world-space axis-aligned bounding boxes of the two
    /// colliders intersect.
    pub(crate) fn query_aabb(&self, aabb: &Aabb) -> bool {
        self.world_aabb.intersects(aabb)
    }
}

impl ColliderWithCache for LocalCollider {
    fn world_aabb(&self) -> &Aabb {
        &self.world_aabb
    }

    fn position(&self) -> &Isometry<f32> {
        &self.position
    }

    fn inner(&self) -> &ObjectCollider {
        &self.object_collider
    }
}

pub struct QueryCollider<'a> {
    inner: &'a ObjectCollider,
    position: Isometry<f32>,
    world_aabb: Aabb,
}

impl<'a> QueryCollider<'a> {
    pub fn new(inner: &'a ObjectCollider, position: Isometry<f32>) -> Self {
        let world_aabb = inner.aabb().transform_by(&position);
        Self {
            inner,
            position,
            world_aabb,
        }
    }
}

impl<'a> ColliderWithCache for QueryCollider<'a> {
    fn world_aabb(&self) -> &Aabb {
        &self.world_aabb
    }

    fn position(&self) -> &Isometry<f32> {
        &self.position
    }

    fn inner(&self) -> &ObjectCollider {
        self.inner
    }
}
