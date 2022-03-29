use bevy::{
    prelude::Mesh,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use glam::{Mat4, Vec3A};
use std::cmp::Ordering;

pub struct Ray {
    origin: Vec3A,
    direction: Vec3A,
}

impl Ray {
    pub fn new<V1: Into<Vec3A>, V2: Into<Vec3A>>(origin: V1, direction: V2) -> Self {
        Self {
            origin: origin.into(),
            direction: direction.into(),
        }
    }

    pub fn origin(&self) -> Vec3A {
        self.origin
    }

    pub fn direction(&self) -> Vec3A {
        self.direction
    }
}

pub struct RayIntersection {
    position: Vec3A,
    distance: f32,
}

impl RayIntersection {
    fn new(position: Vec3A, distance: f32) -> Self {
        if !distance.is_finite() {
            panic!("Got non-finite distance: {}", distance);
        }
        if distance < 0. {
            panic!("Got negative distance: {}", distance);
        }
        Self { position, distance }
    }

    pub fn position(&self) -> Vec3A {
        self.position
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }
}

pub fn ray_mesh_intersection(
    ray: &Ray,
    mesh: &Mesh,
    mesh_to_world: &Mat4,
) -> Option<RayIntersection> {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        panic!("Only TriangleList topology is supported.");
    }
    let vertex_position_atribute = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("Mesh does not contain vertex positions.");
    let vertices: &Vec<[f32; 3]> = match vertex_position_atribute {
        VertexAttributeValues::Float32x3(positions) => positions,
        _ => panic!("Unexpected types in {}", Mesh::ATTRIBUTE_POSITION),
    };

    let world_to_mesh = mesh_to_world.inverse();
    let ray = Ray::new(
        world_to_mesh.transform_point3a(ray.origin()),
        world_to_mesh.transform_vector3a(ray.direction()),
    );
    ray_triangles_intersection(&ray, vertices, mesh.indices()).map(|intersection| {
        RayIntersection::new(
            mesh_to_world.transform_point3a(intersection.position()),
            intersection.distance(),
        )
    })
}

fn ray_triangles_intersection(
    ray: &Ray,
    vertices: &[[f32; 3]],
    indices: Option<&Indices>,
) -> Option<RayIntersection> {
    match indices {
        Some(indices) => match indices {
            Indices::U16(indices) => indices
                .chunks(3)
                .filter_map(|i| {
                    ray_triangle_intersection_triples(
                        ray,
                        vertices[i[0] as usize],
                        vertices[i[1] as usize],
                        vertices[i[2] as usize],
                    )
                })
                .min_by(cmp_by_distance),
            Indices::U32(indices) => indices
                .chunks(3)
                .filter_map(|i| {
                    ray_triangle_intersection_triples(
                        ray,
                        vertices[i[0] as usize],
                        vertices[i[1] as usize],
                        vertices[i[2] as usize],
                    )
                })
                .min_by(cmp_by_distance),
        },
        None => vertices
            .chunks(3)
            .filter_map(|i| ray_triangle_intersection_triples(ray, i[0], i[1], i[2]))
            .min_by(cmp_by_distance),
    }
}

fn cmp_by_distance(a: &RayIntersection, b: &RayIntersection) -> Ordering {
    a.distance().partial_cmp(&b.distance()).unwrap()
}

fn ray_triangle_intersection_triples(
    ray: &Ray,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> Option<RayIntersection> {
    let a = Vec3A::from_slice(&a);
    let b = Vec3A::from_slice(&b);
    let c = Vec3A::from_slice(&c);
    ray_triangle_intersection(ray, a, b, c)
}

fn ray_triangle_intersection(ray: &Ray, a: Vec3A, b: Vec3A, c: Vec3A) -> Option<RayIntersection> {
    let edge1 = a - c;
    let edge2 = c - b;
    let normal = edge1.cross(edge2);
    let intersection = ray_plane_intersection(ray, c, normal)?;

    if edge1.cross(intersection.position() - c).dot(normal) > 0. {
        return None;
    }
    if edge2.cross(intersection.position() - b).dot(normal) > 0. {
        return None;
    }
    if (b - a).cross(intersection.position() - a).dot(normal) > 0. {
        return None;
    }
    Some(intersection)
}

pub fn ray_plane_intersection(ray: &Ray, point: Vec3A, normal: Vec3A) -> Option<RayIntersection> {
    let n_dot_d = normal.dot(ray.direction);
    if n_dot_d.abs() <= f32::EPSILON {
        return None;
    }
    let distance = (point - ray.origin).dot(normal) / n_dot_d;
    if distance < 0. {
        return None;
    }
    Some(RayIntersection::new(
        ray.origin + ray.direction * distance,
        distance,
    ))
}

#[cfg(test)]
mod tests {

    use super::*;
    use bevy::prelude::{shape::Plane, Transform};
    use glam::Vec3;

    #[test]
    fn test_ray_triangle_intersection() {
        let ray = Ray::new(Vec3A::new(0.5, 8.0, 0.1), Vec3A::new(0., -1., 0.));
        let a = Vec3A::new(0.1, 1.44, 0.2);
        let b = Vec3A::new(1.1, 1.44, 0.2);
        let c = Vec3A::new(0.6, 1.44, -0.3);
        let intersection =
            ray_triangle_intersection(&ray, a, b, c).expect("Intersection expected but not found.");
        assert_eq!(intersection.position(), Vec3A::new(0.5, 1.44, 0.1));
        assert_eq!(intersection.distance(), 8.0 - 1.44);
    }

    #[test]
    fn test_ray_mesh_intersection() {
        let ray = Ray::new(Vec3A::new(1., 2., 3.), Vec3A::new(1., -1., 0.));
        let mesh = Mesh::from(Plane { size: 100. });
        let transform = Transform {
            translation: -Vec3::Y,
            ..Default::default()
        };

        let intersection = ray_mesh_intersection(&ray, &mesh, &transform.compute_matrix())
            .expect("Intersection expected but not found");
        assert_eq!(intersection.position(), Vec3A::new(4., -1., 3.));
        assert_eq!(intersection.distance(), 3.);
    }
}
