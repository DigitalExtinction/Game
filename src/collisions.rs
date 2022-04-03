use crate::math::ray::{ray_aabb_intersection, ray_mesh_intersection, Ray, RayIntersection};
use bevy::{
    ecs::{
        query::{FilterFetch, WorldQuery},
        system::SystemParam,
    },
    prelude::{Assets, Entity, GlobalTransform, Handle, Mesh, Query, ResMut},
    render::primitives::Aabb,
};

#[derive(SystemParam)]
pub struct SolidObjects<'w, 's, F>
where
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    objects: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static Aabb,
            &'static Handle<Mesh>,
        ),
        F,
    >,
    meshes: ResMut<'w, Assets<Mesh>>,
}

impl<'w, 's, F> SolidObjects<'w, 's, F>
where
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    pub fn ray_intersection(&self, ray: &Ray) -> Option<(Entity, RayIntersection)> {
        let mut intersection: Option<(Entity, RayIntersection)> = None;

        for (entity, transform, aabb, mesh_handle) in self.objects.iter() {
            let mesh_to_world = transform.compute_matrix();
            if ray_aabb_intersection(ray, aabb, &mesh_to_world).is_none() {
                continue;
            }
            let mesh = self.meshes.get(mesh_handle).unwrap();
            if let Some(candidate) = ray_mesh_intersection(ray, mesh, &mesh_to_world) {
                match &intersection {
                    Some((_, current)) => {
                        if current.distance() > candidate.distance() {
                            intersection = Some((entity, candidate));
                        }
                    }
                    None => intersection = Some((entity, candidate)),
                }
            }
        }

        intersection
    }
}
