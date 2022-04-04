use crate::math::ray::{ray_aabb_intersection, ray_mesh_intersection, Ray, RayIntersection};
use bevy::{
    ecs::{
        query::{FilterFetch, WorldQuery},
        system::SystemParam,
    },
    hierarchy::Children,
    prelude::{Assets, Entity, GlobalTransform, Handle, Mesh, Query, ResMut},
    render::primitives::Aabb,
};

#[derive(SystemParam)]
pub struct SolidObjects<'w, 's, F>
where
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    objects: Query<'w, 's, Entity, F>,
    descendants: Query<'w, 's, &'static Children>,
    intersectable: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static Aabb,
            &'static Handle<Mesh>,
        ),
    >,
    meshes: ResMut<'w, Assets<Mesh>>,
}

impl<'w, 's, F> SolidObjects<'w, 's, F>
where
    F: WorldQuery + Sync + Send + 'static,
    <F as WorldQuery>::Fetch: FilterFetch,
{
    /// Get a closest entity intersecting a given ray. The intersection is
    /// computed against the mesh of the entity and all meshes of descendant
    /// entities.
    pub fn ray_intersection(&self, ray: &Ray) -> Option<(Entity, RayIntersection)> {
        let mut intersection: Option<(Entity, RayIntersection)> = None;
        // (ancestor, entity_to_explore)
        let mut stack: Vec<(Entity, Entity)> = self.objects.iter().map(|e| (e, e)).collect();

        while let Some((ancestor, entity)) = stack.pop() {
            if let Ok(children) = self.descendants.get(entity) {
                for &child in children.iter() {
                    stack.push((ancestor, child));
                }
            }

            if let Ok((transform, aabb, mesh_handle)) = self.intersectable.get(entity) {
                let mesh_to_world = transform.compute_matrix();
                if ray_aabb_intersection(ray, aabb, &mesh_to_world).is_none() {
                    continue;
                }

                let mesh = self.meshes.get(mesh_handle).unwrap();
                if let Some(candidate) = ray_mesh_intersection(ray, mesh, &mesh_to_world) {
                    match &intersection {
                        Some((_, current)) => {
                            if current.distance() > candidate.distance() {
                                intersection = Some((ancestor, candidate));
                            }
                        }
                        None => {
                            intersection = Some((ancestor, candidate));
                        }
                    }
                }
            }
        }

        intersection
    }
}
