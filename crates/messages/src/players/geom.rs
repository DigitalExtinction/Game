#[cfg(feature = "bevy")]
use bevy::transform::components::Transform;
use bincode::{Decode, Encode};
#[cfg(feature = "bevy")]
use glam::Quat;
use glam::{Vec2, Vec3, Vec4};
use nalgebra::{Point2, Point3, Point4, Vector2, Vector3, Vector4};

/// Network representation of translation and rotation. Note that scale is
/// assumed to be always 1.0 along all axes.
#[derive(Debug, Encode, Decode)]
pub struct TransformNet {
    translation: Vec3Net,
    rotation: Vec4Net,
}

#[cfg(feature = "bevy")]
impl From<&Transform> for TransformNet {
    fn from(transform: &Transform) -> Self {
        Self {
            translation: transform.translation.into(),
            rotation: Vec4Net {
                x: transform.rotation.x,
                y: transform.rotation.y,
                z: transform.rotation.z,
                w: transform.rotation.w,
            },
        }
    }
}

#[cfg(feature = "bevy")]
impl From<Transform> for TransformNet {
    fn from(transform: Transform) -> Self {
        Self::from(&transform)
    }
}

#[cfg(feature = "bevy")]
impl From<&TransformNet> for Transform {
    fn from(transform: &TransformNet) -> Self {
        Self {
            translation: transform.translation.into(),
            rotation: Quat::from_vec4(transform.rotation.into()),
            scale: Vec3::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct Vec2Net {
    x: f32,
    y: f32,
}

impl From<Vec2> for Vec2Net {
    fn from(vec: Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<Point2<f32>> for Vec2Net {
    fn from(point: Point2<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Vector2<f32>> for Vec2Net {
    fn from(vector: Vector2<f32>) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
        }
    }
}

impl From<Vec2Net> for Vec2 {
    fn from(vec: Vec2Net) -> Self {
        Self::new(vec.x, vec.y)
    }
}

impl From<Vec2Net> for Point2<f32> {
    fn from(vec: Vec2Net) -> Self {
        Self::new(vec.x, vec.y)
    }
}

impl From<Vec2Net> for Vector2<f32> {
    fn from(vec: Vec2Net) -> Self {
        Self::new(vec.x, vec.y)
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct Vec3Net {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for Vec3Net {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl From<Point3<f32>> for Vec3Net {
    fn from(point: Point3<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}

impl From<Vector3<f32>> for Vec3Net {
    fn from(vector: Vector3<f32>) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }
}

impl From<Vec3Net> for Vec3 {
    fn from(vec: Vec3Net) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl From<Vec3Net> for Point3<f32> {
    fn from(vec: Vec3Net) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl From<Vec3Net> for Vector3<f32> {
    fn from(vec: Vec3Net) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct Vec4Net {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl From<Vec4> for Vec4Net {
    fn from(vec: Vec4) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: vec.w,
        }
    }
}

impl From<Point4<f32>> for Vec4Net {
    fn from(point: Point4<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
            w: point.w,
        }
    }
}

impl From<Vector4<f32>> for Vec4Net {
    fn from(vector: Vector4<f32>) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
            w: vector.w,
        }
    }
}

impl From<Vec4Net> for Vec4 {
    fn from(vec: Vec4Net) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }
}

impl From<Vec4Net> for Point4<f32> {
    fn from(vec: Vec4Net) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }
}

impl From<Vec4Net> for Vector4<f32> {
    fn from(vec: Vec4Net) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }
}
