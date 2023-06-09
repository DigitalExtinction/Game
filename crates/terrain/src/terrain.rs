use ahash::AHashMap;
use bevy::{
    prelude::{Bundle, Component, Mesh, Transform},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::FloatOrd,
};
use de_core::projection::{ToAltitude, ToFlat};
use de_map::size::MapBounds;
use glam::{Vec2, Vec3};
use parry3d::{
    math::Isometry,
    na::{DMatrix, Vector3},
    query::{Ray, RayCast, RayIntersection},
    shape::HeightField,
};

#[derive(Bundle)]
pub struct TerrainBundle {
    transform: Transform,
    terrain: Terrain,
}

impl TerrainBundle {
    pub fn flat(bounds: MapBounds) -> Self {
        let transform = Transform::from_translation(Vec3::from(bounds.aabb().to_msl().center()));
        let size = bounds.size();
        let terrain = Terrain::new(HeightField::new(
            DMatrix::from_row_slice(2, 2, &[0., 0., 0., 0.]),
            Vector3::new(size.x, 1., size.y),
        ));

        Self { transform, terrain }
    }
}

#[derive(Component)]
pub struct Terrain {
    heightfield: HeightField,
}

impl Terrain {
    fn new(heightfield: HeightField) -> Self {
        Self { heightfield }
    }

    pub(crate) fn cast_ray(
        &self,
        m: &Isometry<f32>,
        ray: &Ray,
        max_toi: f32,
    ) -> Option<RayIntersection> {
        self.heightfield
            .cast_ray_and_get_normal(m, ray, max_toi, true)
    }

    pub(crate) fn generate_mesh(&self, translation: Vec3) -> Mesh {
        let translation = translation.to_flat();

        let mut point_to_index: AHashMap<[FloatOrd; 2], u32> = AHashMap::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut positions = Vec::<[f32; 3]>::new();
        let mut normals = Vec::<[f32; 3]>::new();
        let mut uvs = Vec::<[f32; 2]>::new();

        for triangle in self.heightfield.triangles() {
            for point in [triangle.a, triangle.c, triangle.b] {
                let key = [FloatOrd(point.x), FloatOrd(point.z)];
                match point_to_index.get(&key) {
                    Some(&index) => indices.push(index),
                    None => {
                        let index = point_to_index.len() as u32;
                        point_to_index.insert(key, index);
                        indices.push(index);

                        let world = Vec2::new(point.x, point.z).to_msl();
                        positions.push([world.x, point.y, world.z]);
                        normals.push([0., 1., 0.]);
                        uvs.push([point.x + translation.x, point.z + translation.y]);
                    }
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
