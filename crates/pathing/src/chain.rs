//! Implementation of a point linked list with additional methods used for path
//! finding.

use std::rc::Rc;

use de_types::path::Path;
use parry2d::math::Point;

use crate::geometry::{which_side, Side};

/// A linked list of points which keeps track of its length in meters.
pub(crate) struct PointChain {
    prev: Option<Rc<Self>>,
    point: Point<f32>,
    length: f32,
}

impl PointChain {
    /// Creates a new point with a given predecessor.
    ///
    /// # Arguments
    ///
    /// * `chain` - reference to a previous point to be used as an immediate
    ///   predecessor. The reference is cloned with [`Rc::clone`].
    ///
    /// * `point` - extension point. This point must differ from the last point
    ///   in `chain`
    pub(crate) fn extended(chain: &Rc<Self>, point: Point<f32>) -> Self {
        let length = chain.length() + (point - chain.point()).magnitude();
        Self::new(Some(Rc::clone(chain)), point, length)
    }

    /// Creates a new point with no predecessors.
    pub(crate) fn first(point: Point<f32>) -> Self {
        Self::new(None, point, 0.)
    }

    fn new(prev: Option<Rc<Self>>, point: Point<f32>, length: f32) -> Self {
        Self {
            prev,
            point,
            length,
        }
    }

    /// Returns previous point or None if this is the first point.
    pub(crate) fn prev(&self) -> Option<&Rc<Self>> {
        self.prev.as_ref()
    }

    pub(crate) fn point(&self) -> Point<f32> {
        self.point
    }

    /// Returns length of the point chain in meters. It is equal to the sum of
    /// distances of individual points.
    pub(crate) fn length(&self) -> f32 {
        self.length
    }

    /// Returns true if the point has no predecessor.
    pub(crate) fn is_first(&self) -> bool {
        self.prev.is_none()
    }

    /// Returns relative side of a point to `self` from the perspective of the
    /// parent point. Returns `None` if `self` has no parent.
    ///
    /// See [`crate::geometry::which_side`].
    ///
    /// # Panics
    ///
    /// May panic if self is a degenerate point chain or of `point` coincides
    /// with last but one point in self.
    pub(crate) fn which_side(&self, point: Point<f32>) -> Option<Side> {
        self.prev
            .as_ref()
            .map(|p| which_side(p.point(), self.point, point))
    }

    /// Returns an iterator over points in this linked list. The iterator
    /// starts at `self` and traverses all predecessors.
    pub(crate) fn iter(&self) -> Predecessors {
        Predecessors::new(self)
    }

    /// Converts point chain to a path.
    pub(crate) fn to_path(&self) -> Path {
        Path::new(
            self.length(),
            self.iter().map(|t| t.point().into()).collect(),
        )
    }
}

pub(crate) struct Predecessors<'a> {
    chain: Option<&'a PointChain>,
}

impl<'a> Predecessors<'a> {
    fn new(chain: &'a PointChain) -> Self {
        Self { chain: Some(chain) }
    }
}

impl<'a> Iterator for Predecessors<'a> {
    type Item = &'a PointChain;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.chain;
        self.chain = self.chain.and_then(|c| c.prev()).map(Rc::as_ref);
        next
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    #[test]
    fn test_chain() {
        let chain = PointChain::first(Point::new(1., 2.));
        assert!(chain.prev().is_none());
        assert!(chain.is_first());
        assert_eq!(chain.point(), Point::new(1., 2.));
        assert_eq!(chain.length(), 0.);
        let collected: Vec<Point<f32>> = chain.iter().map(|p| p.point()).collect();
        assert_eq!(collected, vec![Point::new(1., 2.)]);

        let chain = PointChain::extended(&Rc::new(chain), Point::new(3., 2.));
        assert!(chain.prev().is_some());
        assert!(!chain.is_first());
        assert_eq!(chain.point(), Point::new(3., 2.));
        assert_eq!(chain.length(), 2.);
        let collected: Vec<Point<f32>> = chain.iter().map(|p| p.point()).collect();
        assert_eq!(collected, vec![Point::new(3., 2.), Point::new(1., 2.)]);
    }

    #[test]
    fn test_which_side() {
        let chain = PointChain::first(Point::new(1., 2.));
        assert!(chain.which_side(Point::new(2., 1.)).is_none());

        let chain = PointChain::extended(&Rc::new(chain), Point::new(3., 2.));
        assert_eq!(chain.which_side(Point::new(2., 1.)).unwrap(), Side::Left);
        assert_eq!(chain.which_side(Point::new(2., 3.)).unwrap(), Side::Right);
    }

    #[test]
    fn test_to_path() {
        let chain = PointChain::extended(
            &Rc::new(PointChain::first(Point::new(1., 2.))),
            Point::new(3., 2.),
        );
        let path = chain.to_path();
        assert_eq!(path.length(), 2.);
        assert_eq!(path.waypoints(), &[Vec2::new(3., 2.), Vec2::new(1., 2.)]);
    }
}
