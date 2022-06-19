use de_core::objects::ObjectType;
use parry3d::{
    bounding_volume::AABB,
    math::{Isometry, Point},
    query::{Ray, RayCast},
    shape::{Shape, TriMesh, TriMeshFlags},
};

use crate::{loader::TriMeshShape, ObjectCache};

pub trait ColliderCache {
    fn get_collider(&self, object_type: ObjectType) -> &ObjectCollider;
}

impl ColliderCache for ObjectCache {
    fn get_collider(&self, object_type: ObjectType) -> &ObjectCollider {
        self.get(object_type).collider()
    }
}

#[derive(Clone)]
pub struct ObjectCollider {
    shape: TriMesh,
}

impl ObjectCollider {
    pub fn new(shape: TriMesh) -> Self {
        Self { shape }
    }

    pub fn compute_aabb(&self) -> AABB {
        self.shape.compute_local_aabb()
    }

    pub fn cast_ray(&self, position: &Isometry<f32>, ray: &Ray, max_toi: f32) -> Option<f32> {
        self.shape.cast_ray(position, ray, max_toi, true)
    }
}

impl From<&TriMeshShape> for ObjectCollider {
    fn from(shape: &TriMeshShape) -> Self {
        let vertices: Vec<Point<f32>> = shape
            .vertices()
            .iter()
            .map(|&[x, y, z]| Point::new(x, y, z))
            .collect();
        let indices = shape.indices().to_owned();
        Self::new(TriMesh::with_flags(
            vertices,
            indices,
            TriMeshFlags::MERGE_DUPLICATE_VERTICES,
        ))
    }
}
