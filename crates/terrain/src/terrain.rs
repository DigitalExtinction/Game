use bevy::{
    prelude::{Bundle, Component, Mesh, Transform},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use de_core::projection::ToMsl;
use de_map::size::MapBounds;
use glam::Vec3;
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

    pub(crate) fn generate_mesh(&self) -> Mesh {
        let aabb = self.heightfield.local_aabb();
        let vertices = [
            ([aabb.mins.x, 0., aabb.mins.z], [0., 1., 0.], [0., 0.]),
            ([aabb.mins.x, 0., aabb.maxs.z], [0., 1., 0.], [0., 1.]),
            ([aabb.maxs.x, 0., aabb.maxs.z], [0., 1., 0.], [1., 1.]),
            ([aabb.maxs.x, 0., aabb.mins.z], [0., 1., 0.], [1., 0.]),
        ];

        let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

        let mut positions = Vec::<[f32; 3]>::new();
        let mut normals = Vec::<[f32; 3]>::new();
        let mut uvs = Vec::<[f32; 2]>::new();
        for (position, normal, uv) in &vertices {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
