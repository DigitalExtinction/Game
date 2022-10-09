use glam::Vec2;

use super::scale::scalar_to_scale;

/// Inclusive parameter bounds of a line.
#[derive(Clone, Copy)]
pub(super) struct Bounds(i32, i32);

impl Bounds {
    /// Computes and returns line parameter bounds given by a zero centered
    /// disc.
    pub(super) fn compute(point: Vec2, dir: Vec2, radius_squared: f32) -> Option<Self> {
        debug_assert!((dir.length_squared() - 1.).abs() < 0.001);
        let b = 2. * point.dot(dir);
        let c = point.length_squared() - radius_squared;

        let discriminant = b.powi(2) - 4. * c;

        if discriminant <= 0. {
            None
        } else {
            let e = -b / 2.;
            let f = discriminant.sqrt() / 2.;
            let min = scalar_to_scale(e - f);
            let max = scalar_to_scale(e + f);
            Some(Self::new(min, max))
        }
    }

    /// Returns new parameter bounds. The bounds are inclusive from both sides.
    pub(super) fn new(min: i32, max: i32) -> Self {
        Self(min, max)
    }

    pub(super) fn contains(&self, value: i32) -> bool {
        self.0 <= value && value <= self.1
    }

    pub(super) fn min(&self) -> i32 {
        self.0
    }

    pub(super) fn max(&self) -> i32 {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute() {
        let bounds = Bounds::compute(Vec2::new(0.5, 1.), Vec2::X, 9.).unwrap();
        assert_eq!(bounds.min(), -3408);
        assert_eq!(bounds.max(), 2384);
        let bounds = Bounds::compute(Vec2::new(0.5, 1.), Vec2::X, 1.);
        assert!(bounds.is_none());
    }

    #[test]
    fn test_min_max_contains() {
        let bounds = Bounds::new(-5, 12);
        assert_eq!(bounds.min(), -5);
        assert_eq!(bounds.max(), 12);
        assert!(bounds.contains(-5));
        assert!(bounds.contains(-1));
        assert!(bounds.contains(12));
        assert!(!bounds.contains(-6));
        assert!(!bounds.contains(13));
    }
}
