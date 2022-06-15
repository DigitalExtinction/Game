use bevy::prelude::Component;
use geo::Polygon;
use parry3d::{bounding_volume::AABB, math::Isometry, query::Ray, shape::Shape};

pub struct EntityShape {
    shape: Box<dyn Shape>,
    /// Position of the shape relative to the entity.
    local_position: Isometry<f32>,
}

impl EntityShape {
    /// Creates a new entity shape.
    ///
    /// # Arguments
    ///
    /// * `shape` - shape of the entity (usually centered at origin).
    ///
    /// * `local_position` - entity-space position of `shape`.
    pub fn new(shape: impl Shape, local_position: Isometry<f32>) -> Self {
        Self {
            shape: Box::new(shape),
            local_position,
        }
    }

    pub fn compute_aabb(&self) -> AABB {
        self.shape.compute_aabb(&self.local_position)
    }

    pub fn cast_ray(
        &self,
        entity_position: &Isometry<f32>,
        ray: &Ray,
        max_toi: f32,
    ) -> Option<f32> {
        let position = entity_position * self.local_position;
        self.shape.cast_ray(&position, ray, max_toi, true)
    }
}

#[derive(Component, Clone)]
pub struct Ichnography {
    bounds: Polygon<f32>,
}

impl Ichnography {
    pub fn new(bounds: Polygon<f32>) -> Self {
        Self { bounds }
    }

    pub fn bounds(&self) -> &Polygon<f32> {
        &self.bounds
    }
}
