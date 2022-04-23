use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use criterion::{criterion_group, criterion_main, Criterion};
use de_game::math::ray::{ray_mesh_intersection, Ray};
use glam::{Mat4, Vec3A};

fn generate_mesh(grid_size: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let scale = 1. / grid_size as f32;
    for row in 0..grid_size {
        for column in 0..grid_size {
            let x1 = column as f32 * scale;
            let x2 = x1 + scale;
            let z1 = row as f32 * scale;
            let z2 = z1 + scale;
            positions.push([x1, 0., z1]);
            uvs.push([x1, z1]);
            normals.push([0., 1., 0.]);
            positions.push([x1, 0., z2]);
            uvs.push([x1, z2]);
            normals.push([0., 1., 0.]);
            positions.push([x2, 0., z2]);
            uvs.push([x2, z2]);
            normals.push([0., 1., 0.]);
            positions.push([x1, 0., z1]);
            uvs.push([x1, z1]);
            normals.push([0., 1., 0.]);
            positions.push([x2, 0., z2]);
            uvs.push([x2, z2]);
            normals.push([0., 1., 0.]);
            positions.push([x2, 0., z1]);
            uvs.push([x2, z1]);
            normals.push([0., 1., 0.]);
        }
    }

    let indices = Indices::U32((0..positions.len()).map(|i| i as u32).collect());
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

fn mesh_intersection_benchmark(c: &mut Criterion) {
    c.bench_function("Ray Mesh Intersection", |b| {
        let ray = Ray::new(Vec3A::Y, -Vec3A::Y);
        let mesh = generate_mesh(100);
        let mesh_to_world = Mat4::from_rotation_z(1.);
        b.iter(|| ray_mesh_intersection(&ray, &mesh, &mesh_to_world))
    });
}

criterion_group!(benches, mesh_intersection_benchmark);
criterion_main!(benches);
