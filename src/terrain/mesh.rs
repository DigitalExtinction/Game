use super::grid::{DiscretePoint, ValueGrid};
use super::rtin::RtinBuilder;
use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use std::collections::HashMap;

fn compute_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    if indices.len() % 3 != 0 {
        panic!("Number of indices is not divisible by 3: {}", indices.len());
    }

    let mut normals = Vec::with_capacity(vertices.len());
    for _ in 0..vertices.len() {
        normals.push([0., 0., 0.]);
    }

    for triangle in indices.chunks(3) {
        let a = vertices[triangle[0] as usize];
        let b = vertices[triangle[1] as usize];
        let c = vertices[triangle[2] as usize];

        // Calculate sides of the triangle:
        let a1 = a[0] - c[0];
        let a2 = a[1] - c[1];
        let a3 = a[2] - c[2];
        let b1 = b[0] - c[0];
        let b2 = b[1] - c[1];
        let b3 = b[2] - c[2];

        // Compute cross product.
        let s1 = a2 * b3 - a3 * b2;
        let s2 = a3 * b1 - a1 * b3;
        let s3 = a1 * b2 - a2 * b1;

        // Add the cross product to existing normals. This will result in
        // weighted normals after normalization.
        for i in 0..3 {
            normals[triangle[i] as usize][0] += s1;
            normals[triangle[i] as usize][1] += s2;
            normals[triangle[i] as usize][2] += s3;
        }
    }

    // Normalize
    for normal in normals.iter_mut() {
        let size = normal.iter().map(|s| s * s).sum::<f32>().sqrt();
        normal[0] /= size;
        normal[1] /= size;
        normal[2] /= size;
    }

    normals
}

fn build_vertices(
    points: &[DiscretePoint],
    elevation_map: &ValueGrid,
    point_size: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut id_to_index: HashMap<u32, usize> = HashMap::new();
    let mut vertices = Vec::new();
    let mut uvs = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let uv_denominator = elevation_map.size() as f32;

    for point in points {
        let vec_id = elevation_map.size() as u32 * point.v + point.u;
        match id_to_index.get(&vec_id) {
            Some(vec_index) => indices.push(*vec_index as u32),
            None => {
                let vec_index = vertices.len();
                id_to_index.insert(vec_id, vec_index);
                indices.push(vec_index as u32);
                vertices.push([
                    point_size * (point.u as f32),
                    elevation_map.value(*point),
                    point_size * (point.v as f32),
                ]);

                // TODO: isn't direction swapped?
                uvs.push([
                    (point.u as f32) / uv_denominator,
                    (point.v as f32) / uv_denominator,
                ]);
            }
        };
    }
    (vertices, uvs, indices)
}

pub fn build_mesh() -> Mesh {
    let mut elevation_map = ValueGrid::with_zeros(17);
    elevation_map.set_value(DiscretePoint { u: 1, v: 1 }, 0.3);

    let point_size = 0.5;
    let max_error = 0.2;

    let points = RtinBuilder::new(&elevation_map, max_error).build();
    let (vertices, uvs, indices) = build_vertices(&points, &elevation_map, point_size);
    let normals = compute_normals(&vertices, &indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
