use bevy::reflect::TypeUuid;
use glam::Vec2;
use parry2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, TypeUuid, Serialize, Deserialize, PartialEq)]
#[uuid = "bbf80d94-c4de-4c7c-9bdc-552ef25aff4e"]
pub struct MapBounds(Vec2);

impl MapBounds {
    /// Create new map bounds spanning a rectangle between -(size / 2.0) and a
    /// (size / 2.0).
    ///
    /// # Panics
    ///
    /// Panics if invalid maximum does not have positive finite coordinates.
    pub fn new(size: Vec2) -> Self {
        let bounds = Self(size / 2.);
        bounds.validate().unwrap();
        bounds
    }

    /// Minimum point of the map.
    pub fn min(&self) -> Vec2 {
        -self.0
    }

    /// Maximum point of the map.
    pub fn max(&self) -> Vec2 {
        self.0
    }

    /// Bounding box of the map.
    pub fn aabb(&self) -> AABB {
        AABB::new(self.min().into(), self.max().into())
    }

    pub fn size(&self) -> Vec2 {
        2. * self.0
    }

    /// Return true if the point lies within map boundaries. Note that map
    /// boundaries are inclusive.
    pub fn contains(&self, point: Vec2) -> bool {
        self.0.cmpge(point.abs()).all()
    }

    pub(crate) fn validate(&self) -> Result<(), MapBoundsValidationError> {
        if !self.0.is_finite() || self.0.cmple(Vec2::ZERO).any() {
            return Err(MapBoundsValidationError { half_size: self.0 });
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
#[error("Map half-size has to be a positive and finite: got ({}, {})", .half_size.x, .half_size.y)]
pub struct MapBoundsValidationError {
    half_size: Vec2,
}

#[cfg(test)]
mod test {
    use parry2d::math::Point;

    use super::*;

    #[test]
    fn test_bounds() {
        let bounds = MapBounds(Vec2::new(2.5, 3.5));
        assert_eq!(
            bounds.aabb(),
            AABB::new(Point::new(-2.5, -3.5), Point::new(2.5, 3.5))
        );
    }

    #[test]
    fn test_contains() {
        let bounds = MapBounds(Vec2::new(2., 3.));
        assert!(bounds.contains(Vec2::ZERO));
        assert!(bounds.contains(Vec2::new(2., 3.)));
        assert!(!bounds.contains(Vec2::new(3., 3.)));
        assert!(!bounds.contains(Vec2::new(f32::INFINITY, 3.)));
        assert!(!bounds.contains(Vec2::new(f32::NEG_INFINITY, 3.)));
        assert!(!bounds.contains(Vec2::new(f32::NAN, 3.)));
    }

    #[test]
    fn test_validate() {
        assert!(MapBounds(Vec2::new(2.5, 3.)).validate().is_ok());
        assert!(MapBounds(Vec2::new(f32::NAN, 2.)).validate().is_err());
        assert!(MapBounds(Vec2::new(f32::INFINITY, 2.)).validate().is_err());
        assert!(MapBounds(Vec2::new(f32::NEG_INFINITY, 2.))
            .validate()
            .is_err());
        assert!(MapBounds(Vec2::new(2., 0.)).validate().is_err());

        let bounds = MapBounds(Vec2::new(-2.5, 3.));
        match bounds.validate() {
            Err(error) => {
                assert_eq!(error.half_size, Vec2::new(-2.5, 3.));
                assert_eq!(
                    format!("{}", error),
                    "Map half-size has to be a positive and finite: got (-2.5, 3)"
                );
            }
            Ok(()) => unreachable!(),
        }
    }
}
