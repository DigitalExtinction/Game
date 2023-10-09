use parry2d::{
    math::Point,
    query::{PointQuery, Ray, RayCast},
    shape::Segment,
};

use crate::geometry::{which_side, RayProjection, Side};

#[derive(Clone)]
pub(crate) struct SegmentInterval {
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
    pub(crate) fn from_projection(segment: Segment, projection: ParamPair, edge_id: u32) -> Self {
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
    pub(crate) fn new(
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
    pub(crate) fn a_corner(&self) -> Option<Point<f32>> {
        if self.is_a_corner {
            Some(self.segment.a)
        } else {
            None
        }
    }

    /// Returns the corner point of the original edge (see [`Self::edge_id()`])
    /// if it corresponds to the endpoint of `self`.
    pub(crate) fn b_corner(&self) -> Option<Point<f32>> {
        if self.is_b_corner {
            Some(self.segment.b)
        } else {
            None
        }
    }

    /// Returns edge ID of the original edge.
    pub(crate) fn edge_id(&self) -> u32 {
        self.edge_id
    }

    pub(crate) fn distance_to_point(&self, point: Point<f32>) -> f32 {
        self.segment.distance_to_local_point(&point, false)
    }

    pub(crate) fn project_point(&self, point: Point<f32>) -> Point<f32> {
        self.segment.project_local_point(&point, false).point
    }

    /// Calculates the cross point of an optimal path from a point `a` to a
    /// point `b` via the interval.
    pub(crate) fn cross(&self, a: Point<f32>, b: Point<f32>) -> SegmentCross {
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
    pub(crate) fn project_onto_segment(
        &self,
        eye: Point<f32>,
        target: Segment,
    ) -> SegmentProjection {
        let ray_a = self.ray_a(eye);
        let ray_b = self.ray_b(eye);
        debug_assert_eq!(ray_a.origin, ray_b.origin);

        let a = RayProjection::calculate(ray_a, target);
        let b = RayProjection::calculate(ray_b, target);

        let side = which_side(ray_a.origin, ray_a.point_at(1.), ray_b.point_at(1.));
        // TODO: for some reason this assert fails due to the ray origin and
        // self.segment lying on a line.
        // debug_assert!(side != Side::Straight || ray_a.dir.dot(&ray_b.dir) < 0.);
        SegmentProjection::new(a, b, side, target.length())
    }

    /// Returns a ray with a given origin and pointing towards the endpoint a
    /// of the interval.
    ///
    /// When `origin` corresponds to the endpoint a, the direction of the
    /// returned ray will correspond to (a - b).
    pub(crate) fn ray_a(&self, origin: Point<f32>) -> Ray {
        Self::endpoint_ray(origin, self.segment.a, self.segment.b)
    }

    /// See [`Self::ray_a()`].
    pub(crate) fn ray_b(&self, origin: Point<f32>) -> Ray {
        Self::endpoint_ray(origin, self.segment.b, self.segment.a)
    }

    fn endpoint_ray(origin: Point<f32>, endpoint: Point<f32>, other_endpoint: Point<f32>) -> Ray {
        let eye = if origin == endpoint {
            other_endpoint
        } else {
            origin
        };
        Ray::new(origin, endpoint - eye)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SegmentCross {
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
    pub(crate) fn point(&self) -> Point<f32> {
        match self {
            Self::Direct(point) => *point,
            Self::Corner(point) => *point,
        }
    }
}

// TODO improve the docs
/// The projection can be looked at as a shadow cast by `self` onto `target`
/// with the source of light placed at `eye`.
#[derive(Clone, Copy)]
pub(crate) struct SegmentProjection {
    a: RayProjection,
    b: RayProjection,
    ray_b_side: Side,
    // TODO rename (everywhere to scale?)
    size: f32,
}

impl SegmentProjection {
    // TODO docs
    fn new(a: RayProjection, b: RayProjection, ray_b_side: Side, size: f32) -> Self {
        Self {
            a,
            b,
            ray_b_side,
            size,
        }
    }

    // TODO document
    pub(crate) fn side_a(&self) -> Option<ParamPair> {
        if self.ray_b_side == Side::Straight {
            return None;
        }

        let first = self.a.parameter().unwrap_or(1.);
        let second = if self.a.endpoint_a_side() == self.ray_b_side {
            1.
        } else {
            0.
        };

        ParamPair::ordered(first, second, self.size)
    }

    // TODO document
    pub(crate) fn side_b(&self) -> Option<ParamPair> {
        if self.ray_b_side == Side::Straight {
            return None;
        }

        let first = self.b.parameter().unwrap_or(1.);
        let second = if self.b.endpoint_a_side() != self.ray_b_side {
            1.
        } else {
            0.
        };

        ParamPair::ordered(first, second, self.size)
    }

    // TODO document
    pub(crate) fn middle(&self) -> Option<ParamPair> {
        if self.ray_b_side == Side::Straight {
            return Some(ParamPair::new(0., 1.));
        }

        match (self.a.parameter(), self.b.parameter()) {
            (Some(a), Some(b)) => ParamPair::ordered(a, b, self.size),
            (None, None) => {
                if self.a.endpoint_a_side() == self.b.endpoint_a_side() {
                    None
                } else {
                    Some(ParamPair::new(0., 1.))
                }
            }
            (Some(first), None) | (None, Some(first)) => {
                let second = if self.a.endpoint_a_side() == self.b.endpoint_a_side() {
                    1.
                } else {
                    0.
                };
                ParamPair::ordered(first, second, self.size)
            }
        }
    }
}

/// Parameters of a (sub-)segment of a line segment.
pub(crate) struct ParamPair(f32, f32);

impl ParamPair {
    // TODO docs
    fn round(parameter: f32, size: f32) -> f32 {
        // Due to the nature of the algorithm, the ray and the segment
        // frequently intersect near one of the endpoints. To avoid rounding issues,

        let scaled = parameter * size;

        // TODO use constants (negligible distance)
        if scaled.abs() < 0.01 {
            0.
        } else if (size - scaled).abs() < 0.01 {
            1.
        } else {
            parameter
        }
    }

    // TODO document
    fn ordered(a: f32, b: f32, size: f32) -> Option<Self> {
        let a = Self::round(a, size);
        let b = Self::round(b, size);

        if a < b {
            Some(Self::new(a, b))
        } else if a > b {
            Some(Self::new(b, a))
        } else {
            None
        }
    }

    fn new(a: f32, b: f32) -> Self {
        debug_assert!(0. <= a);
        debug_assert!(a < b);
        debug_assert!(b <= 1.);
        Self(a, b)
    }

    /// Apply the parameters on the parent line segment and return the
    /// (sub-)segment.
    fn apply(&self, segment: Segment) -> Segment {
        debug_assert!(segment.length() > 0.);
        let dir = segment.scaled_direction();
        Segment::new(
            if self.0 == 0. {
                // To avoid rounding errors around corners.
                segment.a
            } else {
                segment.a + self.0 * dir
            },
            if self.1 == 1. {
                segment.b
            } else {
                segment.a + self.1 * dir
            },
        )
    }

    /// Returns true if the first parameter coincides with endpoint a of the
    /// parent line segment.
    fn includes_corner_a(&self) -> bool {
        self.0 == 0.
    }

    /// Returns true if the first parameter coincides with endpoint b of the
    /// parent line segment.
    fn includes_corner_b(&self) -> bool {
        self.1 == 1.
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use super::*;
    use crate::geometry::SimpleSide;

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
    fn test_project_onto_segment() {
        let interval = SegmentInterval::new(
            Segment::new(Point::new(2., 4.), Point::new(2., 1.)),
            true,
            true,
            0,
        );

        let projection = interval.project_onto_segment(
            Point::new(0., 4.),
            Segment::new(Point::new(4., 2.), Point::new(4., 10.)),
        );
        assert_eq!(projection.ray_b_side, Side::Left);
        assert_eq!(projection.a.parameter().unwrap(), 0.25);
        assert_eq!(projection.a.endpoint_a_side(), SimpleSide::Left);
        assert!(projection.b.parameter().is_none());
        assert_eq!(projection.b.endpoint_a_side(), SimpleSide::Right);

        let projection = interval.project_onto_segment(
            Point::new(0., 4.),
            Segment::new(Point::new(4., 10.), Point::new(4., 2.)),
        );
        assert_eq!(projection.a.parameter().unwrap(), 0.75);
        assert_eq!(projection.a.endpoint_a_side(), SimpleSide::Right);
        assert!(projection.b.parameter().is_none());
        assert_eq!(projection.a.endpoint_a_side(), SimpleSide::Right);
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

    #[test]
    fn test_ray() {
        let interval = SegmentInterval::new(
            Segment::new(Point::new(2., 4.), Point::new(2., 1.)),
            true,
            true,
            0,
        );

        let ray = interval.ray_a(Point::new(0.5, 2.5));
        assert_eq!(ray.origin, Point::new(0.5, 2.5));
        assert_eq!(ray.dir, Vector2::new(1.5, 1.5));

        let ray = interval.ray_b(Point::new(0.5, 2.5));
        assert_eq!(ray.origin, Point::new(0.5, 2.5));
        assert_eq!(ray.dir, Vector2::new(1.5, -1.5));

        let ray = interval.ray_a(Point::new(2., 4.));
        assert_eq!(ray.origin, Point::new(2., 4.));
        assert_eq!(ray.dir, Vector2::new(0., 3.));
    }
}
