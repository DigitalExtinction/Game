//! Tools and struts for working with paths on the surface of a map.

use bevy::prelude::Component;
use de_types::path::Path;
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
            let mut remainder_length = remainder.length();

            if self.current == start {
                remainder_length /= CURRENT_SEGMENT_BIAS;
            }

            if remainder_length > amount {
                advancement += (amount / remainder_length) * remainder;
                break;
            }

            self.current -= 1;
            advancement = segment_end;
            amount -= remainder_length;
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
}
