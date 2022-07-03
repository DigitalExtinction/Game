//! Tools and struts for working with paths on the surface of a map.

#[cfg(debug_assertions)]
use approx::assert_abs_diff_eq;
use bevy::prelude::Component;
use glam::Vec2;

use crate::PathTarget;

#[derive(Component)]
pub struct PathResult {
    path: Path,
    target: PathTarget,
}

impl PathResult {
    pub(crate) fn new(path: Path, target: PathTarget) -> Self {
        Self { path, target }
    }

    pub fn path_mut(&mut self) -> &mut Path {
        &mut self.path
    }

    pub fn target(&self) -> PathTarget {
        self.target
    }
}

/// A path on the map defined by a sequence of way points. Start and target
/// position are included.
pub struct Path {
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

    /// Returns the original length of the path in meters.
    pub(crate) fn length(&self) -> f32 {
        self.length
    }

    /// Returns a sequence of the remaining path way points. The last way point
    /// corresponds to the start of the path and vice versa.
    pub fn waypoints(&self) -> &[Vec2] {
        self.waypoints.as_slice()
    }

    /// Advances the path by one. Returns true if the path is empty.
    pub fn advance(&mut self) -> bool {
        self.waypoints.pop();
        self.waypoints.is_empty()
    }

    /// Returns a path shortened by `amount` from the end. Returns None
    /// `amount` is longer than the path.
    pub(crate) fn truncated(mut self, mut amount: f32) -> Option<Self> {
        if amount == 0. {
            return Some(self);
        } else if amount >= self.length {
            return None;
        }

        for i in 1..self.waypoints.len() {
            debug_assert!(self.length > 0.);
            debug_assert!(amount > 0.);

            let last = self.waypoints[i - 1];
            let preceeding = self.waypoints[i];

            let diff = last - preceeding;
            let diff_len = diff.length();

            if diff_len < amount {
                self.length -= diff_len;
                amount -= diff_len;
            } else if diff_len > amount {
                self.length -= amount;
                let mut waypoints = Vec::with_capacity(self.waypoints.len() - i + 1);
                let fraction = (diff_len - amount) / diff_len;
                waypoints.push(preceeding + fraction * diff);
                waypoints.extend_from_slice(&self.waypoints[i..]);
                return Some(Self::new(self.length, waypoints));
            } else if diff_len == amount {
                self.length -= amount;
                let mut waypoints = Vec::with_capacity(self.waypoints.len() - i);
                waypoints.extend_from_slice(&self.waypoints[i..]);
                return Some(Self::new(self.length, waypoints));
            }
        }

        // This might happen due to rounding errors. Otherwise, this code
        // should be unreachable.
        None
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

    #[test]
    fn test_truncated() {
        let path = Path::new(
            8.,
            vec![Vec2::new(1., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)],
        )
        .truncated(2.)
        .unwrap();
        assert_eq!(path.length(), 6.);
        assert_eq!(path.waypoints(), vec![Vec2::new(3., 2.), Vec2::new(3., 8.)]);

        let path = Path::new(
            8.,
            vec![Vec2::new(1., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)],
        )
        .truncated(1.)
        .unwrap();
        assert_eq!(path.length(), 7.);
        assert_eq!(
            path.waypoints(),
            vec![Vec2::new(2., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)]
        );

        assert!(Path::new(
            8.,
            vec![Vec2::new(1., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)],
        )
        .truncated(8.)
        .is_none());

        assert!(Path::new(
            8.,
            vec![Vec2::new(1., 2.), Vec2::new(3., 2.), Vec2::new(3., 8.)],
        )
        .truncated(10.)
        .is_none());
    }
}
