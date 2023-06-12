use parry3d::{
    bounding_volume::Aabb,
    math::{Isometry, Point},
    query::{intersection_test, PointQuery, Ray, RayCast},
    shape::{Shape, TriMesh, TriMeshFlags},
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ObjectCollider {
    aabb: Aabb,
    shape: TriMesh,
}

impl ObjectCollider {
    fn new(aabb: Aabb, shape: TriMesh) -> Self {
        debug_assert!(shape.pseudo_normals().is_some());
        Self { aabb, shape }
    }

    pub fn aabb(&self) -> Aabb {
        self.aabb
    }

    pub fn cast_ray(&self, position: &Isometry<f32>, ray: &Ray, max_toi: f32) -> Option<f32> {
        self.shape.cast_ray(position, ray, max_toi, true)
    }

    pub fn intersects(
        &self,
        position: &Isometry<f32>,
        rhs: &Self,
        rhs_position: &Isometry<f32>,
    ) -> bool {
        // This must be here since intersection_test() tests only for collider
        // surface to collider surface intersection. We need to return true
        // even if one is fully contained in the other.
        if self.contains_first_vertex(position, rhs, rhs_position) {
            return true;
        }
        if rhs.contains_first_vertex(rhs_position, self, position) {
            return true;
        }
        intersection_test(position, &self.shape, rhs_position, &rhs.shape).unwrap()
    }

    /// Returns true if `self` contains first vertex of `rhs`.
    fn contains_first_vertex(
        &self,
        position: &Isometry<f32>,
        rhs: &Self,
        rhs_position: &Isometry<f32>,
    ) -> bool {
        let any_rhs_point = rhs_position.transform_point(&rhs.shape.vertices()[0]);
        self.shape.contains_point(position, &any_rhs_point)
    }
}

impl From<TriMesh> for ObjectCollider {
    fn from(mesh: TriMesh) -> Self {
        Self::new(mesh.compute_local_aabb(), mesh)
    }
}

impl TryFrom<ColliderSerde> for ObjectCollider {
    type Error = anyhow::Error;

    fn try_from(shape: ColliderSerde) -> Result<Self, Self::Error> {
        let vertices: Vec<Point<f32>> = shape
            .vertices
            .iter()
            .map(|&[x, y, z]| Point::new(x, y, z))
            .collect();
        let trimesh = TriMesh::with_flags(
            vertices,
            shape.indices,
            TriMeshFlags::MERGE_DUPLICATE_VERTICES | TriMeshFlags::ORIENTED,
        );
        Ok(Self::from(trimesh))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ColliderSerde {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

#[cfg(test)]
mod tests {
    use parry3d::{
        math::{Isometry, Vector},
        shape::{Cuboid, TriMesh, TriMeshFlags},
    };

    use crate::ObjectCollider;

    #[test]
    fn test_intersects() {
        assert!(collider(1.).intersects(
            &Isometry::translation(10., 0., 0.),
            &collider(10.),
            &Isometry::translation(10., 0., 0.),
        ));
        assert!(collider(10.).intersects(
            &Isometry::translation(10., 0., 0.),
            &collider(1.),
            &Isometry::translation(10., 0., 0.),
        ));
        assert!(collider(1.).intersects(
            &Isometry::translation(10., 0., 0.),
            &collider(1.),
            &Isometry::translation(10., 0., 0.),
        ));
        assert!(collider(1.).intersects(
            &Isometry::translation(9.5, 0., 0.),
            &collider(1.),
            &Isometry::translation(10., 0., 0.),
        ));
        assert!(!collider(1.).intersects(
            &Isometry::translation(-10., 0., 0.),
            &collider(1.),
            &Isometry::translation(10., 0., 0.),
        ));
    }

    fn collider(size: f32) -> ObjectCollider {
        let cube = Cuboid::new(Vector::new(size, size, size));
        let (vertices, indices) = cube.to_trimesh();
        ObjectCollider::from(TriMesh::with_flags(
            vertices,
            indices,
            TriMeshFlags::ORIENTED,
        ))
    }
}
