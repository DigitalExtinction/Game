use parry2d::{
    math::Point,
    query::{PointQuery, Ray, RayCast},
    shape::Segment,
};

use crate::segmentproj::{ParamPair, SegmentOnSegmentProjection};

#[derive(Clone)]
pub(super) struct SegmentInterval {
    segment: Segment,
    is_a_corner: bool,
    is_b_corner: bool,
    edge_id: u32,
}

impl SegmentInterval {
    /// Creates a new interval from projection parameters.
    ///
    /// # Arguments
    ///
    /// * `segment` - an original segment. Projection parameters correspond to
    ///   this segment.
    ///
    /// * `projection` - parameters of endpoints a and b of the interval to be
    ///   created.
    ///
    /// * `edge_id` - ID of the original edge / segment.
    ///
    /// # Panics
    ///
    /// May panic if projection parameters are not between 0 and 1 (inclusive)
    /// or if first projection parameter is larger or equal to the second
    /// projection parameter.
    pub(super) fn from_projection(segment: Segment, projection: ParamPair, edge_id: u32) -> Self {
        Self::new(
            projection.apply(segment),
            projection.includes_corner_a(),
            projection.includes_corner_b(),
            edge_id,
        )
    }

    /// Creates a new segment interval.
    ///
    /// # Panics
    ///
    /// May panic if `segment` has zero length.
    pub(super) fn new(
        segment: Segment,
        is_a_corner: bool,
        is_b_corner: bool,
        edge_id: u32,
    ) -> Self {
        debug_assert!(segment.length() > 0.);
        Self {
            segment,
            is_a_corner,
            is_b_corner,
            edge_id,
        }
    }

    /// Returns the corner point of the original edge (see [`Self::edge_id()`])
    /// if it corresponds to the endpoint of `self`.
    pub(super) fn a_corner(&self) -> Option<Point<f32>> {
        if self.is_a_corner {
            Some(self.segment.a)
        } else {
            None
        }
    }

    /// Returns the corner point of the original edge (see [`Self::edge_id()`])
    /// if it corresponds to the endpoint of `self`.
    pub(super) fn b_corner(&self) -> Option<Point<f32>> {
        if self.is_b_corner {
            Some(self.segment.b)
        } else {
            None
        }
    }

    /// Returns edge ID of the original edge.
    pub(super) fn edge_id(&self) -> u32 {
        self.edge_id
    }

    pub(super) fn distance_to_point(&self, point: Point<f32>) -> f32 {
        self.segment.distance_to_local_point(&point, false)
    }

    pub(super) fn project_point(&self, point: Point<f32>) -> Point<f32> {
        self.segment.project_local_point(&point, false).point
    }

    /// Calculates the cross point of an optimal path from a point `a` to a
    /// point `b` via the interval.
    pub(super) fn cross(&self, a: Point<f32>, b: Point<f32>) -> SegmentCross {
        let ray = Ray::new(a, b - a);
        let direct_cross = self
            .segment
            .cast_local_ray(&ray, 1., false)
            .map(|param| ray.point_at(param));

        match direct_cross {
            Some(point) => SegmentCross::Direct(point),
            None => {
                let dist_a = (self.segment.a - a).magnitude() + (self.segment.a - b).magnitude();
                let dist_b = (self.segment.b - a).magnitude() + (self.segment.b - b).magnitude();

                if dist_a <= dist_b {
                    SegmentCross::Corner(self.segment.a)
                } else {
                    SegmentCross::Corner(self.segment.b)
                }
            }
        }
    }

    /// Returns projection of self onto a target segment from a given
    /// perspective.
    ///
    /// # Arguments
    ///
    /// * `eye` - projection perspective.
    ///
    /// * `target` - self is projected onto this target.
    pub(super) fn project_onto_segment(
        &self,
        eye: Point<f32>,
        target: Segment,
    ) -> SegmentOnSegmentProjection {
        SegmentOnSegmentProjection::construct(eye, self.segment, target)
    }
}

#[derive(Clone, Copy)]
pub(super) enum SegmentCross {
    /// The crossed line segment intersects with the line segment between the
    /// points `a` and `b`.
    Direct(Point<f32>),
    /// The crossed line segment does not intersect with the line segment
    /// between the points `a` and `b` and thus the optimal path traverses an
    /// endpoint of the crossed line segment.
    Corner(Point<f32>),
}

impl SegmentCross {
    /// Returns the crossing point.
    pub(super) fn point(&self) -> Point<f32> {
        match self {
            Self::Direct(point) => *point,
            Self::Corner(point) => *point,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::segmentproj::ParamPair;

    #[test]
    fn test_from_projection() {
        let interval = SegmentInterval::from_projection(
            Segment::new(Point::new(2., 4.), Point::new(2., 0.)),
            ParamPair::new(0.25, 0.5),
            3,
        );
        assert_eq!(interval.segment.a, Point::new(2., 3.));
        assert_eq!(interval.segment.b, Point::new(2., 2.));
        assert_eq!(interval.a_corner(), None);
        assert_eq!(interval.b_corner(), None);
        assert_eq!(interval.edge_id(), 3);

        let interval = SegmentInterval::from_projection(
            Segment::new(Point::new(2., 4.), Point::new(2., 0.)),
            ParamPair::new(0., 1.),
            7,
        );
        assert_eq!(interval.segment.a, Point::new(2., 4.));
        assert_eq!(interval.segment.b, Point::new(2., 0.));
        assert_eq!(interval.a_corner().unwrap(), Point::new(2., 4.));
        assert_eq!(interval.b_corner().unwrap(), Point::new(2., 0.));
        assert_eq!(interval.edge_id(), 7);
    }

    #[test]
    fn test_project_point() {
        let interval = SegmentInterval::new(
            Segment::new(Point::new(2., 4.), Point::new(2., 1.)),
            true,
            true,
            0,
        );
        assert_eq!(
            interval.project_point(Point::new(8., 2.)),
            Point::new(2., 2.)
        );
        assert_eq!(
            interval.project_point(Point::new(8., 10.)),
            Point::new(2., 4.)
        );
    }

    #[test]
    fn test_left_corner() {
        let a = Point::new(-1., 1.);
        let b = Point::new(1., 1.);
        let c = Point::new(-3., 0.);
        let eye = Point::new(-2., 2.);

        let target = Segment::new(b, c);

        let parameters = [
            ((a, b), (a, c)),
            ((b, a), (a, c)),
            ((a, b), (c, a)),
            ((b, a), (c, a)),
        ];
        for ((aa, ab), (ba, bb)) in parameters {
            let interval = SegmentInterval::new(Segment::new(aa, ab), true, true, 0);
            let proj = interval.project_onto_segment(eye, Segment::new(ba, bb));
            assert!(proj.middle().is_none());

            let pair = match (proj.side_a(), proj.side_b()) {
                (Some(pair), None) => pair,
                (None, Some(pair)) => pair,
                _ => unreachable!("The segment fully behind one corner."),
            };

            assert!(pair.includes_corner_a());
            assert!(pair.includes_corner_b());

            let result = pair.apply(target);
            assert!(result.a == b || result.b == b);
            assert!(result.a == c || result.b == c);
        }
    }

    #[test]
    fn test_right_corner() {
        let a = Point::new(-1., 1.);
        let b = Point::new(1., 1.);
        let c = Point::new(3., 0.);
        let eye = Point::new(2., 2.);

        let target = Segment::new(b, c);

        let parameters = [
            ((a, b), (b, c)),
            ((b, a), (b, c)),
            ((a, b), (c, b)),
            ((b, a), (c, b)),
        ];
        for ((aa, ab), (ba, bb)) in parameters {
            let interval = SegmentInterval::new(Segment::new(aa, ab), true, true, 0);
            let proj = interval.project_onto_segment(eye, Segment::new(ba, bb));
            assert!(proj.middle().is_none());

            let pair = match (proj.side_a(), proj.side_b()) {
                (Some(pair), None) => pair,
                (None, Some(pair)) => pair,
                _ => unreachable!("The segment fully behind one corner."),
            };

            assert!(pair.includes_corner_a());
            assert!(pair.includes_corner_b());

            let result = pair.apply(target);
            assert!(result.a == b || result.b == b);
            assert!(result.a == c || result.b == c);
        }
    }

    #[test]
    fn test_eye_on_endpoint() {
        let a = Point::new(-1., 1.);
        let b = Point::new(1., 1.);
        let c = Point::new(3., 0.);
        let eye = b;

        let target = Segment::new(b, c);

        let parameters = [
            ((a, b), (b, c)),
            ((b, a), (b, c)),
            ((a, b), (c, b)),
            ((b, a), (c, b)),
        ];
        for ((aa, ab), (ba, bb)) in parameters {
            let interval = SegmentInterval::new(Segment::new(aa, ab), true, true, 0);
            let proj = interval.project_onto_segment(eye, Segment::new(ba, bb));
            assert!(proj.side_a().is_none());
            assert!(proj.side_b().is_none());

            let pair = proj.middle().unwrap();
            assert!(pair.includes_corner_a());
            assert!(pair.includes_corner_b());

            let result = pair.apply(target);
            assert!(result.a == b || result.b == b);
            assert!(result.a == c || result.b == c);
        }
    }
}
