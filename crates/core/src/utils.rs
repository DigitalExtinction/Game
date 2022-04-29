use glam::{Vec2, Vec3};

/// Trait for conversion of various geometrical objects to their 3D equivalents
/// placed to mean sea level.
pub trait ToMsl {
    type InMsl;

    fn to_msl(self) -> Self::InMsl;
}

impl ToMsl for Vec2 {
    type InMsl = Vec3;

    fn to_msl(self) -> Vec3 {
        Vec3::new(self.x, 0., self.y)
    }
}

impl ToMsl for Vec3 {
    type InMsl = Vec3;

    fn to_msl(self) -> Vec3 {
        Vec3::new(self.x, 0., self.z)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vec2() {
        let vec = Vec2::new(10.5, 15.5);
        let vec3 = vec.to_msl();
        assert_eq!(vec3.x, 10.5);
        assert_eq!(vec3.y, 0.);
        assert_eq!(vec3.z, 15.5);
    }
}
