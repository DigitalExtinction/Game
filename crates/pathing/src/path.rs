//! Tools and struts for working with paths on the surface of a map.

#[cfg(debug_assertions)]
use approx::assert_abs_diff_eq;
use bevy::prelude::Component;
use glam::Vec2;

const CURRENT_SEGMENT_BIAS: f32 = 4.;

/// A path on the map which may be followed by an object or a group of objects.
#[derive(Component)]
pub struct ScheduledPath {
    path: Path,
    current: usize,
}

impl ScheduledPath {
    /// Creates a new path schedule.
    ///
    /// # Panics
    ///
    /// May panic if `path` has less than two points.
    pub(crate) fn new(path: Path) -> Self {
        debug_assert!(path.waypoints().len() >= 2);
        let current = path.waypoints().len() - 1;
        Self { path, current }
    }

    /// Returns the final point of the path schedule.
    pub fn destination(&self) -> Vec2 {
        self.path.waypoints()[0]
    }

    /// Advances the path schedule by a given distance and returns the
    /// corresponding point on the path.
    ///
    /// # Arguments
    ///
    /// * `position` - position of the object(s) tracking this path. It is used
    ///   as a base for the path advancement.
    ///
    /// * `amount` - advancement distance in meters. The advancement is
    ///   computed in the mode which tries to keep the object withing `amount`
    ///   meters from the scheduled path.
    ///
    ///   Advancement along current path line segment is multiplied by a factor
    ///   larger than one.
    pub fn advance(&mut self, position: Vec2, amount: f32) -> Vec2 {
        if self.current == 0 {
            return self.path.waypoints()[0];
        }

        let (mut advancement, projection_factor) = self.projection(position);
        let mut amount = amount - position.distance(advancement);

        if amount <= 0. {
            return advancement;
        }

        let start = if projection_factor > 0. {
            self.current
        } else {
            self.current + 1
        };
        while self.current > 0 {
            let segment_end = self.path.waypoints()[self.current - 1];
            let remainder = segment_end - advancement;
            let mut remainder_lenght = remainder.length();

            if self.current == start {
                remainder_lenght /= CURRENT_SEGMENT_BIAS;
            }

            if remainder_lenght > amount {
                advancement += (amount / remainder_lenght) * remainder;
                break;
            }

            self.current -= 1;
            advancement = segment_end;
            amount -= remainder_lenght;
        }

        advancement
    }

    /// Returns a point and segment fraction on current segment of the path
    /// closest to a given `position`.
    ///
    /// This method cannot be called if only one (last) point remains to be
    /// reached.
    ///
    /// # Panics
    ///
    /// Panics if it is called when only last point remains.
    fn projection(&self, position: Vec2) -> (Vec2, f32) {
        let start = self.path.waypoints()[self.current];
        let end = self.path.waypoints()[self.current - 1];
        let start_to_end = end - start;

        let factor = (start_to_end / start_to_end.length_squared())
            .dot(position - start)
            .clamp(0., 1.);
        (factor * start_to_end + start, factor)
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

    /// Returns the length of the path in meters.
    pub(crate) fn length(&self) -> f32 {
        self.length
    }

    /// Returns a sequence of the remaining path way points. The last way point
    /// corresponds to the start of the path and vice versa.
    pub(crate) fn waypoints(&self) -> &[Vec2] {
        self.waypoints.as_slice()
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
    fn test_schedule_advance() {
        let mut schedule = ScheduledPath::new(Path::new(
            7.,
            vec![Vec2::new(4., 6.), Vec2::new(4., 1.), Vec2::new(2., 1.)],
        ));
        assert!(
            schedule
                .advance(Vec2::new(2.5, 1.1), 0.2)
                .distance(Vec2::new(2.9, 1.0))
                < 0.001
        );
        assert!(
            schedule
                .advance(Vec2::new(3.4, 1.), 1.)
                .distance(Vec2::new(4.0, 1.85))
                < 0.001
        );
        // Cannon return a point before an already reached segment.
        assert_eq!(
            schedule.advance(Vec2::new(2.1, 1.), 1.),
            Vec2::new(4.0, 1.0)
        );
    }

    #[test]
    fn test_schedule_project() {
        let schedule = ScheduledPath::new(Path::new(
            9.071,
            vec![Vec2::new(5., 8.), Vec2::new(4., 1.), Vec2::new(2., 1.)],
        ));
        assert_eq!(
            schedule.projection(Vec2::new(3.8, 5.)),
            (Vec2::new(3.8, 1.), 0.9)
        );
        assert_eq!(
            schedule.projection(Vec2::new(-2., 3.)),
            (Vec2::new(2., 1.), 0.)
        );
        assert_eq!(
            schedule.projection(Vec2::new(7., 8.)),
            (Vec2::new(4., 1.), 1.)
        );
    }

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
