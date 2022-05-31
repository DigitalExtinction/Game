//! Various low level geometrical operations.

use parry2d::{math::Point, shape::Segment};

/// Reorients the segment so that end point `a` appears on the left side of end
/// point `b` from the perspective of `eye`.
///
/// # Panics
///
/// May panic if `eye` coincides with end points of the segment.
pub(crate) fn orient(eye: Point<f32>, segment: Segment) -> Segment {
    if which_side(eye, segment.a, segment.b) == Side::Left {
        Segment::new(segment.b, segment.a)
    } else {
        segment
    }
}

/// Returns the side at which point `new` appears relative to point `old` from
/// the perspective of `eye`.
///
/// Returns [`Side::Straight`] if `eye` lies on line segment with end points
/// `old` and `new`.
///
/// # Panics
///
/// May panic if `eye` coincides with `old` or `new`.
pub(crate) fn which_side(eye: Point<f32>, old: Point<f32>, new: Point<f32>) -> Side {
    debug_assert!(Point::from(old - eye) != Point::origin());
    debug_assert!(Point::from(new - eye) != Point::origin());
    let perp: f32 = (eye - old).perp(&(eye - new));
    if perp < 0. {
        Side::Left
    } else if perp > 0. {
        Side::Right
    } else {
        Side::Straight
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Straight,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orient() {
        let eye = Point::new(-100., -200.);
        let segment_a = Segment::new(Point::new(1., 2.), Point::new(3., 8.));
        let segment_b = Segment::new(Point::new(3., 8.), Point::new(1., 2.));

        assert_eq!(orient(eye, segment_a), segment_a);
        assert_eq!(orient(eye, segment_b), segment_a);

        assert_eq!(
            orient(
                Point::new(-450.0, -950.0),
                Segment::new(
                    Point::new(18.612133, -18.612133),
                    Point::new(-500.0, -1000.)
                )
            ),
            Segment::new(
                Point::new(18.612133, -18.612133),
                Point::new(-500.0, -1000.),
            )
        );
    }

    #[test]
    fn test_which_side() {
        let eye = Point::new(-1., 2.);
        let a = Point::new(5., -6.);
        let b = Point::new(5., 6.);
        let c = Point::new(-5., 6.);
        let d = Point::new(-5., -6.);

        assert_eq!(which_side(eye, a, b), Side::Right);
        assert_eq!(which_side(eye, b, c), Side::Right);
        assert_eq!(which_side(eye, c, d), Side::Right);
        assert_eq!(which_side(eye, d, a), Side::Right);
        assert_eq!(which_side(eye, b, a), Side::Left);
        assert_eq!(which_side(eye, c, b), Side::Left);
        assert_eq!(which_side(eye, d, c), Side::Left);
        assert_eq!(which_side(eye, a, d), Side::Left);
        assert_eq!(which_side(eye, a, a), Side::Straight);
        assert_eq!(which_side(Point::origin(), a, 0.5 * a), Side::Straight);
    }
}
