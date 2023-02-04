use bevy::{ecs::system::SystemParam, prelude::*};
use parry3d::query::Ray;

#[derive(SystemParam)]
pub(crate) struct ScreenRay<'w, 's> {
    cameras: Query<'w, 's, (&'static Transform, &'static Camera), With<Camera3d>>,
}

impl<'w, 's> ScreenRay<'w, 's> {
    /// Returns line of sight of a point on the screen.
    ///
    /// The ray originates on the near plane of the projection frustum.
    ///
    /// # Arguments
    ///
    /// * `point` - normalized coordinates (between [-1., -1.] and [1., 1.]) of
    ///   a point on the screen.
    pub(crate) fn ray(&self, point: Vec2) -> Ray {
        let (camera_transform, camera) = self.cameras.single();
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let ray_origin = ndc_to_world.project_point3(point.extend(1.));
        let ray_direction = (ray_origin - camera_transform.translation).normalize();
        Ray::new(ray_origin.into(), ray_direction.into())
    }
}
