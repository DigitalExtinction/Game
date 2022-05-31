//! Tools and struts for working with paths on the surface of a map.

#[cfg(debug_assertions)]
use approx::assert_abs_diff_eq;
use bevy::prelude::Component;
use glam::Vec2;

/// A path on the map defined by a sequence of way points. Start and target
/// position are included.
#[derive(Component)]
pub(crate) struct Path {
    length: f32,
    waypoints: Vec<Vec2>,
}

impl Path {
    /// Creates a path on line `from` -> `to`.
    pub(crate) fn straight<P: Into<Vec2>>(from: P, to: P) -> Self {
        let waypoints = vec![to.into(), from.into()];
        Self::new(waypoints[0].distance(waypoints[1]), waypoints)
    }

    /// Creates a new path.
    ///
    /// # Panics
    ///
    /// May panic if sum of distances of `waypoints` is not equal to provided
    /// `length`.
    pub(crate) fn new(length: f32, waypoints: Vec<Vec2>) -> Self {
        #[cfg(debug_assertions)]
        {
            assert_abs_diff_eq!(
                waypoints
                    .windows(2)
                    .map(|pair| (pair[1] - pair[0]).length())
                    .sum::<f32>(),
                length,
                epsilon = 0.001,
            );
        }
        Self { length, waypoints }
    }

    /// Returns length of the path in meters.
    pub(crate) fn length(&self) -> f32 {
        self.length
    }

    /// Returns complete sequence of the path way points. The last way point
    /// corresponds to the start of the path and vice versa.
    #[allow(dead_code)]
    pub(crate) fn waypoints(&self) -> &[Vec2] {
        self.waypoints.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let path = Path::new(
            8.,
            vec![Vec2::new(1., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)],
        );
        assert_eq!(path.length(), 8.);
        assert_eq!(path.waypoints().len(), 3);

        let path = Path::straight(Vec2::new(10., 11.), Vec2::new(22., 11.));
        assert_eq!(path.length(), 12.);
        assert_eq!(path.waypoints().len(), 2);
    }
}
