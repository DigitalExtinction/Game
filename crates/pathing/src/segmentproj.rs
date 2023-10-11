use parry2d::{math::Point, query::Ray, shape::Segment};

use crate::geometry::{which_side, RayProjection, Side};

/// Projection of a line segment onto another line segment from the perspective
/// of an eye (a point). The projection can be looked at as a shadow cast by
/// the one segment onto the other segment with the source of light placed at
/// the eye.
pub(super) struct SegmentOnSegmentProjection {
    side_a: Option<ParamPair>,
    middle: Option<ParamPair>,
    side_b: Option<ParamPair>,
}

impl SegmentOnSegmentProjection {
    pub(super) fn construct(eye: Point<f32>, source: Segment, target: Segment) -> Self {
        let target_length = target.length();

        if eye == source.a || eye == source.b {
            return Self::new(None, Some(ParamPair::new(0., 1.)), None);
        }

        let ray_a = Ray::new(eye, source.a - eye);
        let ray_b = Ray::new(eye, source.b - eye);
        debug_assert_eq!(ray_a.origin, ray_b.origin);

        let ray_a_proj = RayProjection::calculate(ray_a, target);
        let ray_b_proj = RayProjection::calculate(ray_b, target);
        let ray_b_side = which_side(ray_a.origin, ray_a.point_at(1.), ray_b.point_at(1.));

        let side_a = Self::construct_a(ray_a_proj, ray_b_side, target_length);
        let middle = Self::construct_middle(ray_a_proj, ray_b_proj, target_length);
        let side_b = Self::construct_b(ray_b_proj, ray_b_side, target_length);

        Self::new(side_a, middle, side_b)
    }

    fn construct_a(
        ray_a_proj: RayProjection,
        ray_b_side: Side,
        target_length: f32,
    ) -> Option<ParamPair> {
        let param = ray_a_proj.parameter().unwrap_or(1.);
        let corner = if ray_a_proj.endpoint_a_side() == ray_b_side {
            1.
        } else {
            0.
        };

        ParamPair::normalized(param, corner, target_length)
    }

    fn construct_b(
        ray_b_proj: RayProjection,
        ray_b_side: Side,
        target_length: f32,
    ) -> Option<ParamPair> {
        let param = ray_b_proj.parameter().unwrap_or(1.);
        let corner = if ray_b_proj.endpoint_a_side() != ray_b_side {
            1.
        } else {
            0.
        };

        ParamPair::normalized(param, corner, target_length)
    }

    fn construct_middle(
        ray_a_proj: RayProjection,
        ray_b_proj: RayProjection,
        target_length: f32,
    ) -> Option<ParamPair> {
        match (ray_a_proj.parameter(), ray_b_proj.parameter()) {
            (Some(a), Some(b)) => ParamPair::normalized(a, b, target_length),
            (None, None) => {
                if ray_a_proj.endpoint_a_side() == ray_b_proj.endpoint_a_side() {
                    None
                } else {
                    Some(ParamPair::new(0., 1.))
                }
            }
            (Some(param), None) | (None, Some(param)) => {
                let corner = if ray_a_proj.endpoint_a_side() == ray_b_proj.endpoint_a_side() {
                    1.
                } else {
                    0.
                };
                ParamPair::normalized(param, corner, target_length)
            }
        }
    }

    fn new(
        side_a: Option<ParamPair>,
        middle: Option<ParamPair>,
        side_b: Option<ParamPair>,
    ) -> Self {
        assert!(side_a.is_some() || middle.is_some() || side_b.is_some());
        Self {
            side_a,
            middle,
            side_b,
        }
    }

    /// Non-visible part of the target line segment adjacent to endpoint a.
    /// This is None when all of target is visible.
    pub(super) fn side_a(&self) -> Option<ParamPair> {
        self.side_a
    }

    /// Visible part of the target line segment. This is None in None if no
    /// point of the target line segment is visible (from eye via the source
    /// line segment).
    pub(super) fn middle(&self) -> Option<ParamPair> {
        self.middle
    }

    /// Non-visible part of the target line segment adjacent to endpoint b.
    /// This is None when all of target is visible.
    pub(super) fn side_b(&self) -> Option<ParamPair> {
        self.side_b
    }
}

/// Parameters of a (sub-)segment of a line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ParamPair(f32, f32);

impl ParamPair {
    /// Round parameters very close to 0 or 1 to exact 0 or 1.
    ///
    /// # Arguments
    ///
    /// * `parameter` - parameter to be rounded.
    ///
    /// * `scale` - size of the line segment to take into account for the
    ///   rounding. The large the scale the less aggressive the rounding.
    fn round(parameter: f32, scale: f32) -> f32 {
        // Due to the nature of the algorithm, the ray and the segment
        // frequently intersect near one of the endpoints. To avoid rounding
        // issues, this rounding method must be used.

        let scaled = parameter * scale;
        if scaled.abs() < 0.01 {
            0.
        } else if (scale - scaled).abs() < 0.01 {
            1.
        } else {
            parameter
        }
    }

    /// Creates a normalized (sub-)segment parameter pair. The resulting pair
    /// is ordered (i.e. ordering of the first two arguments does not matter)
    /// and rounded (to avoid precision issues).
    ///
    /// None is returned in the case when the resulting interval contains only
    /// a single point.
    ///
    /// # Arguments
    ///
    /// * `a` - first projection parameter. A number between 0. and 1.
    ///   Arguments `a` and `b` may be swapped.
    ///
    /// * `b` - see `a`.
    ///
    /// * `scale` - size of the corresponding line segment. It is used for
    ///   proper parameter rounding.
    fn normalized(a: f32, b: f32, scale: f32) -> Option<Self> {
        let a = Self::round(a, scale);
        let b = Self::round(b, scale);

        if a < b {
            Some(Self::new(a, b))
        } else if a > b {
            Some(Self::new(b, a))
        } else {
            None
        }
    }

    pub(super) fn new(a: f32, b: f32) -> Self {
        debug_assert!(0. <= a);
        debug_assert!(a < b);
        debug_assert!(b <= 1.);
        Self(a, b)
    }

    /// Apply the parameters on the parent line segment and return the
    /// (sub-)segment.
    pub(super) fn apply(&self, segment: Segment) -> Segment {
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
    pub(super) fn includes_corner_a(&self) -> bool {
        self.0 == 0.
    }

    /// Returns true if the first parameter coincides with endpoint b of the
    /// parent line segment.
    pub(super) fn includes_corner_b(&self) -> bool {
        self.1 == 1.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct() {
        let projection = SegmentOnSegmentProjection::construct(
            Point::new(0., 4.),
            Segment::new(Point::new(2., 4.), Point::new(2., 1.)),
            Segment::new(Point::new(4., 2.), Point::new(4., 10.)),
        );
        assert_eq!(projection.side_a(), Some(ParamPair::new(0.25, 1.)));
        assert_eq!(projection.middle(), Some(ParamPair::new(0., 0.25)));
        assert!(projection.side_b().is_none());

        let projection = SegmentOnSegmentProjection::construct(
            Point::new(0., 4.),
            Segment::new(Point::new(2., 4.), Point::new(2., 1.)),
            Segment::new(Point::new(4., 10.), Point::new(4., 2.)),
        );
        assert_eq!(projection.side_a(), Some(ParamPair::new(0., 0.75)));
        assert_eq!(projection.middle(), Some(ParamPair::new(0.75, 1.)));
        assert!(projection.side_b().is_none());
    }
}
