//! This module implements projections to mean seal level (MSL) plane of
//! various 3D objects and mappings between 3D world space and 2D map
//! coordinate system.

use glam::{Vec2, Vec3};
use nalgebra::{Const, OPoint};
use parry2d::{bounding_volume::Aabb as Aabb2D, math::Point as Point2D};
use parry3d::{bounding_volume::Aabb as Aabb3D, math::Point as Point3D};

/// Trait for conversion of various geometrical objects to their 3D equivalents
/// placed to mean sea level.
pub trait ToMsl<Msl> {
    fn to_msl(self) -> Msl;
}

impl ToMsl<Vec3> for Vec2 {
    fn to_msl(self) -> Vec3 {
        Vec3::new(self.x, 0., -self.y)
    }
}

impl ToMsl<Vec3> for Vec3 {
    fn to_msl(self) -> Vec3 {
        Vec3::new(self.x, 0., self.z)
    }
}

impl ToMsl<Aabb3D> for Aabb2D {
    fn to_msl(self) -> Aabb3D {
        Aabb3D::new(
            Point3D::new(self.mins.x, 0., -self.maxs.y),
            Point3D::new(self.maxs.x, 0., -self.mins.y),
        )
    }
}

/// Transformation between 3D world coordinates and 2D map coordinates.
pub trait ToFlat<Flat> {
    fn to_flat(self) -> Flat;
}

impl ToFlat<Vec2> for Vec3 {
    fn to_flat(self) -> Vec2 {
        Vec2::new(self.x, -self.z)
    }
}

impl ToFlat<Vec2> for OPoint<f32, Const<3>> {
    fn to_flat(self) -> Vec2 {
        Vec2::new(self.x, -self.z)
    }
}

impl ToFlat<Aabb2D> for Aabb3D {
    fn to_flat(self) -> Aabb2D {
        Aabb2D::new(
            Point2D::new(self.mins.x, -self.maxs.z),
            Point2D::new(self.maxs.x, -self.mins.z),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_msl() {
        let vec = Vec2::new(10.5, 15.5);
        let vec3 = vec.to_msl();
        assert_eq!(vec3.x, 10.5);
        assert_eq!(vec3.y, 0.);
        assert_eq!(vec3.z, -15.5);
    }

    #[test]
    fn test_to_flat() {
        let vec = Vec3::new(1., 2., 3.);
        assert_eq!(vec.to_flat(), Vec2::new(1., -3.));
    }
}
