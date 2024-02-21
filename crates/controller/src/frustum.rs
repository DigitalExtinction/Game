use std::f32::consts::PI;

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::primitives::{Frustum, HalfSpace},
};
use de_core::screengeom::ScreenRect;

#[derive(SystemParam)]
pub(crate) struct ScreenFrustum<'w, 's> {
    camera: Query<'w, 's, (&'static Transform, &'static Projection), With<Camera3d>>,
}

impl<'w, 's> ScreenFrustum<'w, 's> {
    /// Returns a frustum corresponding to the visible area from a screen
    /// rectangle.
    ///
    /// The near and far planes of the frustum correspond to the near and far
    /// plane of the camera projection.
    ///
    /// # Panics
    ///
    /// * If there is not exactly one `Camera3d` in the world with `Transform`
    ///   and `Projection` components.
    ///
    /// * If the camera doesn't have perspective projection.
    pub(crate) fn rect(&self, rect: ScreenRect) -> Frustum {
        let (transform, projection) = self.camera.single();

        let projection = match projection {
            Projection::Perspective(p) => p,
            _ => panic!(
                "Frustum of a screen rectangle can be computed only for \
                 cameras with perspective projection."
            ),
        };

        debug_assert!(projection.fov < PI);
        debug_assert!(projection.fov > 0.);

        let mut half_spaces = [HalfSpace::default(); 6];

        let y_max = (0.5 * projection.fov).tan();
        let maxs = [y_max * projection.aspect_ratio, y_max];
        let edges = rect.as_array();
        for i in 0..4 {
            let signum = if i % 2 == 0 { 1. } else { -1. };
            let mut norm = [0.; 3];
            norm[i / 2] = signum;
            norm[2] = signum * maxs[i / 2] * edges[i];
            let norm = transform.rotation * Vec3::from_array(norm);
            half_spaces[i] = HalfSpace::new(norm.extend(-transform.translation.dot(norm)));
        }

        let forward = transform.forward();
        let near_dist = -forward.dot(transform.translation + projection.near * forward);
        let far_dist = -forward.dot(transform.translation + projection.far * forward);
        half_spaces[4] = HalfSpace::new(forward.extend(near_dist));
        half_spaces[5] = HalfSpace::new(-forward.extend(far_dist));

        Frustum { half_spaces }
    }
}
