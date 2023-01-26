//! Implementation of funnel algorithm based on linked lists of points and
//! incremental funnel extension API. The API makes the algorithm usable &
//! efficient when used from graph traversal algorithms.

use std::rc::Rc;

use parry2d::{math::Point, shape::Segment};

use crate::{chain::PointChain, geometry::Side, path::Path};

/// The funnel consists of a tail and left & right bounds.
///
/// The tail and bounds are represented by [`crate::chain::PointChain`], thus
/// the funnel can be cheaply cloned, expanded and used in graph traversal
/// algorithms.
///
/// The left and right bounds are represented by a list of points (going
/// backwards from last funnel expansion to the tip of the tail) such that line
/// segments in between them represent gradually closing funnel.
///
/// The tail is represented by a list of points where the funnel was already
/// "closed", id est area where space between bounds narrowed to or below 0.
///
/// No operation on the funnel affects other funnels sharing parts of the point
/// lists with bounds and tail.
#[derive(Clone)]
pub(crate) struct Funnel {
    tail: Rc<PointChain>,
    left: Rc<PointChain>,
    right: Rc<PointChain>,
}

impl Funnel {
    /// Creates a new funnel with a single point.
    pub(crate) fn new(start: Point<f32>) -> Self {
        Self {
            tail: Rc::new(PointChain::first(start)),
            left: Rc::new(PointChain::first(start)),
            right: Rc::new(PointChain::first(start)),
        }
    }

    /// Creates a new funnel where `a` represents side bound on `side` and `b`
    /// represents the opposing bound.
    ///
    /// # Panics
    ///
    /// Panics if `side` is not [`Side::Left`] or [`Side::Right`].
    fn from_sides(side: Side, tail: Rc<PointChain>, a: Rc<PointChain>, b: Rc<PointChain>) -> Self {
        let (left, right) = match side {
            Side::Left => (a, b),
            Side::Right => (b, a),
            _ => panic!("Only Left and Right sides are accepted, got: {side:?}"),
        };
        Self { tail, left, right }
    }

    pub(crate) fn tail(&self) -> &PointChain {
        Rc::as_ref(&self.tail)
    }

    pub(crate) fn left(&self) -> &PointChain {
        Rc::as_ref(&self.left)
    }

    pub(crate) fn right(&self) -> &PointChain {
        Rc::as_ref(&self.right)
    }

    /// Returns the full shortest path inside the funnel to point `by`.
    pub fn closed(&self, by: Point<f32>) -> Path {
        let closed = self
            .extended_by_point(Side::Left, by)
            .extended_by_point(Side::Right, by);

        let left_count = closed.left().iter().count();
        let right_count = closed.right().iter().count();
        debug_assert!(left_count <= 2);
        debug_assert!(right_count <= 2);

        // due to rounding errors, tail might not contain the very last point
        // (i.e. `by`).
        if (left_count + right_count) == 2 {
            closed.tail().to_path()
        } else {
            PointChain::extended(&closed.tail, by).to_path()
        }
    }

    /// Returns a new funnel which is an extension of `self` by a line segment.
    ///
    /// It is supposed that `by.a` is on the left side from the point of view
    /// of the middle of the last expansion segment.
    pub fn extended(&self, by: Segment) -> Self {
        self.extended_by_point(Side::Left, by.a)
            .extended_by_point(Side::Right, by.b)
    }

    /// Returns a new funnel with a side bound of the funnel extended &
    /// modified by a point.
    ///
    /// In the case that the point narrows down the funnel, the operation
    /// results in the removal of an ending portion of the side bound.
    ///
    /// In case that the side bound gets narrowed down beyond the opposing side
    /// bound, the "closed" portion of the funnel is moved to the tail. See
    /// [`close`].
    fn extended_by_point(&self, side: Side, by: Point<f32>) -> Self {
        let (chain, opposing) = self.sides(side);

        if chain.point() == by {
            self.clone()
        } else if chain.which_side(by).map(|s| s == side).unwrap_or(true) {
            self.extended_side(side, by)
        } else {
            let first_to_remove = chain
                .iter()
                .take_while(|b| b.which_side(by).map(|s| s != side).unwrap_or(false))
                .last()
                // At least one item was taken, otherwise previous if else
                // would not be skipped.
                .unwrap();
            let last_to_keep = first_to_remove.prev().unwrap();
            let chain = Rc::new(PointChain::extended(last_to_keep, by));

            if last_to_keep.is_first() {
                let (tail, chain, opposing) =
                    close(side, Rc::clone(&self.tail), chain, Rc::clone(opposing));
                Self::from_sides(side, tail, chain, opposing)
            } else {
                Self::from_sides(side, Rc::clone(&self.tail), chain, Rc::clone(opposing))
            }
        }
    }

    /// Returns a new funnel with a side bound extended by a point. The point
    /// is simple appended on the tip of the side bound.
    fn extended_side(&self, side: Side, by: Point<f32>) -> Self {
        let (chain, opposing) = self.sides(side);
        let extended = Rc::new(PointChain::extended(chain, by));
        Self::from_sides(side, Rc::clone(&self.tail), extended, Rc::clone(opposing))
    }

    fn sides(&self, side: Side) -> (&Rc<PointChain>, &Rc<PointChain>) {
        match side {
            Side::Left => (&self.left, &self.right),
            Side::Right => (&self.right, &self.left),
            _ => panic!("Only Left and Right sides are accepted, got: {side:?}"),
        }
    }
}

/// Moves "closed" part of `opposing` to the tail.
///
/// # Arguments
///
/// * `side` - Side of `chain` bound.
///
/// * `tail` - tail to be expanded by the closed points of `chain`.
///
/// * `chain` - a side bound which "closes" `opposing`.
///
/// * `opposing` - a side bound to be closed.
fn close(
    side: Side,
    tail: Rc<PointChain>,
    chain: Rc<PointChain>,
    opposing: Rc<PointChain>,
) -> (Rc<PointChain>, Rc<PointChain>, Rc<PointChain>) {
    debug_assert!(side == Side::Left || side == Side::Right);

    let by = chain.point();
    let keep = match opposing
        .iter()
        .position(|b| b.which_side(by).map(|s| s != side).unwrap_or(false))
    {
        Some(index) => index,
        None => return (tail, chain, opposing),
    };

    let bounds: Vec<Point<f32>> = opposing.iter().map(|b| b.point()).collect();

    let mut tail = tail;
    // First item in side chains is equal to tail point. It must be skipped
    // here.
    for &point in bounds[keep..].iter().rev() {
        if tail.point() != point {
            tail = Rc::new(PointChain::extended(&tail, point));
        }
    }

    let mut chain = Rc::new(PointChain::first(tail.point()));
    if chain.point() != by {
        chain = Rc::new(PointChain::extended(&chain, by));
    }

    let mut opposing = Rc::new(PointChain::first(tail.point()));
    for &point in bounds[..keep].iter().rev() {
        opposing = Rc::new(PointChain::extended(&opposing, point));
    }

    (tail, chain, opposing)
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use glam::Vec2;

    use super::*;

    #[test]
    fn test_funnel() {
        let point_a = Point::new(1., 1.);
        let point_b = Point::new(2., 0.);
        let point_c = Point::new(2., 2.);
        let point_d = Point::new(3., 1.);
        let point_e = Point::new(2.5, 3.);
        let point_f = Point::new(1.1, 3.);
        let point_g = Point::new(1.7, 4.);
        let point_h = Point::new(2., 5.);
        let point_j = Point::new(2.5, 4.5);

        let funnel = Funnel::new(point_a)
            .extended(Segment::new(point_b, point_c))
            .extended(Segment::new(point_d, point_c))
            .extended(Segment::new(point_e, point_c))
            .extended(Segment::new(point_e, point_f))
            .extended(Segment::new(point_g, point_f))
            .extended(Segment::new(point_g, point_h))
            .extended(Segment::new(point_g, point_j));

        let path: Vec<Point<f32>> = funnel.tail().iter().map(|p| p.point()).collect();
        assert_abs_diff_eq!(funnel.tail().length(), 3.436, epsilon = 0.001);
        assert_eq!(path, vec![point_g, point_c, point_a]);
    }

    #[test]
    fn test_close() {
        // Funnel opening to the right from the point of view of point a.
        let point_a = Point::new(1., 2.);
        let opposing_a = Rc::new(PointChain::first(point_a));
        let point_b = Point::new(2., 3.);
        let opposing_b = Rc::new(PointChain::extended(&opposing_a, point_b));
        let point_c = Point::new(3., 5.);
        let opposing_c = Rc::new(PointChain::extended(&opposing_b, point_c));
        let point_d = Point::new(4., 8.);
        let opposing_d = Rc::new(PointChain::extended(&opposing_c, point_d));

        // Point is to the left from all part of the chain, should not close.
        let (tail, chain, opposing) = close(
            Side::Left,
            Rc::new(PointChain::first(Point::new(1., 2.))),
            Rc::new(PointChain::first(Point::new(5., -1.))),
            Rc::clone(&opposing_d),
        );
        assert_eq!(tail.to_path().waypoints(), &[Vec2::new(1., 2.)]);
        assert_eq!(chain.to_path().waypoints(), &[Vec2::new(5., -1.)]);
        assert_eq!(
            opposing.to_path().waypoints(),
            &[
                Vec2::new(4., 8.),
                Vec2::new(3., 5.),
                Vec2::new(2., 3.),
                Vec2::new(1., 2.),
            ]
        );

        // Point is to the right from all but last two points.
        let (tail, chain, opposing) = close(
            Side::Left,
            Rc::new(PointChain::first(Point::new(1., 2.))),
            Rc::new(PointChain::first(Point::new(5., 8.9))),
            Rc::clone(&opposing_d),
        );
        assert_eq!(
            tail.to_path().waypoints(),
            &[Vec2::new(2., 3.), Vec2::new(1., 2.)]
        );
        assert_eq!(
            chain.to_path().waypoints(),
            &[Vec2::new(5., 8.9), Vec2::new(2., 3.)]
        );
        assert_eq!(
            opposing.to_path().waypoints(),
            &[Vec2::new(4., 8.), Vec2::new(3., 5.), Vec2::new(2., 3.)]
        );

        // Point is to the right from all points.
        let (tail, chain, opposing) = close(
            Side::Left,
            Rc::new(PointChain::first(Point::new(1., 2.))),
            Rc::new(PointChain::first(Point::new(5., 13.))),
            Rc::clone(&opposing_d),
        );
        assert_eq!(
            tail.to_path().waypoints(),
            &[
                Vec2::new(4., 8.),
                Vec2::new(3., 5.),
                Vec2::new(2., 3.),
                Vec2::new(1., 2.),
            ]
        );
        assert_eq!(
            chain.to_path().waypoints(),
            &[Vec2::new(5., 13.), Vec2::new(4., 8.)]
        );
        assert_eq!(opposing.to_path().waypoints(), &[Vec2::new(4., 8.)]);
    }
}
