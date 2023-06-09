use bevy::{
    prelude::{GlobalTransform, Transform},
    render::primitives::{Aabb, Frustum, Sphere},
};
use glam::Vec3;
use parry3d::bounding_volume::Aabb as AabbP;

/// See [`intersects_bevy`].
pub fn intersects_parry(frustum: &Frustum, transform: Transform, aabb: &AabbP) -> bool {
    let transform = GlobalTransform::from(transform);
    let aabb = Aabb::from_min_max(Vec3::from(aabb.mins), Vec3::from(aabb.maxs));
    intersects_bevy(frustum, &transform, &aabb)
}

/// Returns true if object space `aabb` transformer by `transform` intersects
/// the given `frustum`.
///
/// # Arguments
///
/// * `frustum` - frustum to be tested against.
///
/// * `transform` - transformation of the tested object.
///
/// * `aabb` - object space AABB.
pub fn intersects_bevy(frustum: &Frustum, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    let model = transform.compute_matrix();
    let model_sphere = Sphere {
        center: model.transform_point3a(aabb.center),
        radius: transform.radius_vec3a(aabb.half_extents),
    };

    frustum.intersects_sphere(&model_sphere, false)
        && frustum.intersects_obb(aabb, &model, false, true)
}
