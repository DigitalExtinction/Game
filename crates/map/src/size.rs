use bevy::prelude::Resource;
use glam::Vec2;
use parry2d::bounding_volume::Aabb;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum size of a side of the map in meters.
pub const MAX_MAP_SIZE: f32 = 8000.;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Resource)]
pub struct MapBounds(Vec2);

impl MapBounds {
    /// Create new map bounds spanning a rectangle between -(size / 2.0) and a
    /// (size / 2.0).
    ///
    /// # Panics
    ///
    /// * If the size coordinates are not finite and positive.
    /// * If the size is too large.
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
    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.min().into(), self.max().into())
    }

    pub fn size(&self) -> Vec2 {
        2. * self.0
    }

    /// Return true if the point lies within map boundaries. Note that map
    /// boundaries are inclusive.
    pub fn contains(&self, point: Vec2) -> bool {
        self.0.cmpge(point.abs()).all()
    }

    /// Projects a point from relative space to the map flat coordinates.
    ///
    /// # Arguments
    ///
    /// * `point` - relative point on the map between (0, 0) and (1, 1). Point
    ///   (0, 0) corresponds to the south-west corner.
    pub fn rel_to_abs(&self, point: Vec2) -> Vec2 {
        self.min() + point * self.size()
    }

    pub(crate) fn validate(&self) -> Result<(), MapBoundsValidationError> {
        if !self.0.is_finite() || self.0.cmple(Vec2::ZERO).any() {
            return Err(MapBoundsValidationError::Invalid(self.0));
        }

        let max = Vec2::splat(0.5 * MAX_MAP_SIZE);
        if self.0.cmpgt(max).any() {
            return Err(MapBoundsValidationError::TooLarge { max, value: self.0 });
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MapBoundsValidationError {
    #[error("map half-size has to be a positive and finite: got ({}, {})", .0.x, .0.y)]
    Invalid(Vec2),
    #[error("map half-size ({}, {}) is larger than maximum ({}, {})", .value.x, .value.y, .max.x, .max.y )]
    TooLarge { max: Vec2, value: Vec2 },
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
            Aabb::new(Point::new(-2.5, -3.5), Point::new(2.5, 3.5))
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

        let invalid_bounds = MapBounds(Vec2::new(-2.5, 3.));
        match invalid_bounds.validate() {
            Err(error) => {
                match error {
                    MapBoundsValidationError::Invalid(size) => {
                        assert_eq!(size, Vec2::new(-2.5, 3.));
                    }
                    _ => unreachable!("Wrong error returned."),
                }

                assert_eq!(
                    format!("{error}"),
                    "map half-size has to be a positive and finite: got (-2.5, 3)"
                );
            }
            Ok(()) => unreachable!(),
        }

        let too_large_bounds = MapBounds(Vec2::new(10., 99999.));
        match too_large_bounds.validate() {
            Err(error) => {
                match error {
                    MapBoundsValidationError::TooLarge { value, max } => {
                        assert_eq!(max, Vec2::new(4000., 4000.));
                        assert_eq!(value, Vec2::new(10., 99999.));
                    }
                    _ => unreachable!("Wrong error returned."),
                }

                assert_eq!(
                    format!("{error}"),
                    "map half-size (10, 99999) is larger than maximum (4000, 4000)"
                );
            }
            Ok(()) => unreachable!(),
        }
    }
}
