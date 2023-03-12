//! Various low level geometrical operations.

use parry2d::{math::Point, query::Ray, shape::Segment};

/// Returns the side at which point `new` appears relative to point `old` from
/// the perspective of `eye`.
///
/// Returns [`Side::Straight`] if `eye` lies on line segment with end points
/// `old` and `new`.
///
/// # Panics
///
/// May panic if `eye` coincides with `old`.
pub(crate) fn which_side(eye: Point<f32>, old: Point<f32>, new: Point<f32>) -> Side {
    debug_assert!(Point::from(old - eye) != Point::origin());
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

/// Projection of a ray onto a line segment.
#[derive(Clone, Copy)]
pub(crate) struct RayProjection {
    parameter: Option<f32>,
    endpoint_a_side: SimpleSide,
}

impl RayProjection {
    pub(crate) fn calculate(ray: Ray, target: Segment) -> Self {
        let segment_dir = target.scaled_direction();

        let origin_diff = target.a - ray.origin;
        let ray_perp_origin = ray.dir.perp(&origin_diff);
        let ray_perp_dir = ray.dir.perp(&segment_dir);
        let dir_perp_origin = segment_dir.perp(&origin_diff);

        // TODO constant
        // This is true when the ray is parallel with the segment.
        let is_parallel = ray_perp_dir.abs() < 0.0001;
        // This is true when the ray points away from the line given by the
        // segment.
        let is_behind = dir_perp_origin * ray_perp_dir > 0.;

        let parameter = if is_parallel || is_behind {
            None
        } else {
            let parameter = -ray_perp_origin / ray_perp_dir;
            if (0. ..=1.).contains(&parameter) {
                Some(parameter)
            } else {
                None
            }
        };

        let endpoint_a_side = if ray_perp_origin < 0. {
            SimpleSide::Left
        } else if ray_perp_origin > 0. {
            SimpleSide::Right
        } else if ray.dir.perp(&(target.b - ray.origin)) > 0. {
            // When ray goes through endpoint A (or directly away from it), we
            // pretend that the endpoint lies at the opposite site than
            // endpoint B so that the ray "crosses" the segment.
            SimpleSide::Left
        } else {
            SimpleSide::Right
        };

        Self::new(parameter, endpoint_a_side)
    }

    pub(crate) fn new(parameter: Option<f32>, endpoint_a_side: SimpleSide) -> Self {
        #[cfg(debug_assertions)]
        if let Some(parameter) = parameter {
            assert!(parameter.is_finite());
            assert!(0. <= parameter);
            assert!(parameter <= 1.);
        }
        Self {
            parameter,
            endpoint_a_side,
        }
    }

    /// A value between 0 and 1 (inclusive). The parameter is None if the ray
    /// does not intersect the segment.
    ///
    /// The intersection point is given by `segment.a + (segment.b - segment.a)
    /// * parameter`.
    pub(crate) fn parameter(&self) -> Option<f32> {
        self.parameter
    }

    /// Side of endpoint a of the segment relative to the ray.
    ///
    /// In the case that endpoint lies on the line given by the ray, it is
    /// assumed that endpoint a lies on the opposite site than endpoint b.
    pub(crate) fn endpoint_a_side(&self) -> SimpleSide {
        self.endpoint_a_side
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SimpleSide {
    Left,
    Right,
}

impl PartialEq<Side> for SimpleSide {
    fn eq(&self, other: &Side) -> bool {
        match self {
            Self::Left => *other == Side::Left,
            Self::Right => *other == Side::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use super::*;

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

    #[test]
    fn test_ray_projection() {
        let segment = Segment::new(Point::new(3., 1.), Point::new(1., 3.));

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(1., 1.)), segment);
        assert_eq!(0.5, proj.parameter().unwrap());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Left);

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(2., 2.)), segment);
        assert_eq!(0.5, proj.parameter().unwrap());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Left);

        let proj =
            RayProjection::calculate(Ray::new(Point::new(2., 1.), Vector2::new(1., 0.)), segment);
        assert_eq!(0., proj.parameter().unwrap());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Left);

        let proj = RayProjection::calculate(
            Ray::new(Point::new(2., 1.), Vector2::new(1., -0.5)),
            segment,
        );
        assert!(proj.parameter().is_none());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Right);

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(1., -1.)), segment);
        assert!(proj.parameter().is_none());

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(-1., 1.)), segment);
        assert!(proj.parameter().is_none());

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(0., -3.)), segment);
        assert!(proj.parameter().is_none());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Right);

        let proj =
            RayProjection::calculate(Ray::new(Point::origin(), Vector2::new(-3., 0.)), segment);
        assert!(proj.parameter().is_none());
        assert_eq!(proj.endpoint_a_side(), SimpleSide::Left);
    }
}
