use std::ops::Mul;

use glam::IVec2;

use super::scale::{scalar_div_to_scale, scaled_div_floor, vec_from_scale};

/// An oriented line defined by a point and direction.
#[derive(Clone, Copy)]
pub(super) struct Line {
    point: IVec2,
    dir: IVec2,
}

impl Line {
    /// Creates a new line defined by a point and a vector with the line
    /// direction.
    ///
    /// # Arguments
    ///
    /// * `point` - a point on the line.
    ///
    /// * `dir` - direction of the line.
    pub(super) fn new(point: IVec2, dir: IVec2) -> Self {
        Self { point, dir }
    }

    pub(super) fn point(&self) -> IVec2 {
        self.point
    }

    pub(super) fn dir(&self) -> IVec2 {
        self.dir
    }

    /// Returns perpendicular dot product between relative position of `point`
    /// to a point on `self` and direction of `self`.
    fn side(&self, point: IVec2) -> i32 {
        self.dir.perp_dot(point - self.point)
    }

    pub(super) fn projection(&self, point: IVec2) -> i32 {
        scalar_div_to_scale(self.dir.dot(point - self.point))
    }

    /// Returns [`Signum::Positive`] if `point` lies on the left side of
    /// `self`, returns [`Signum::Negative`] if `point` lies on the right.
    /// Returns 0 otherwise.
    pub(super) fn side_signum(&self, point: IVec2) -> Signum {
        self.side(point).signum().try_into().unwrap()
    }

    /// Computes and returns intersection between `self` and `other`.
    pub(super) fn intersection(&self, other: Line) -> Option<LineLineIntersection> {
        let denominator = self.dir.perp_dot(other.dir);
        let primary_side = other.side(self.point);

        if denominator == 0 {
            if primary_side == 0 {
                Some(LineLineIntersection::Coincidental)
            } else {
                None
            }
        } else {
            let secondary_side = self.side(other.point);
            Some(LineLineIntersection::Point(Intersection {
                side_signum: secondary_side.signum().try_into().unwrap(),
                dir_signum: denominator.signum().try_into().unwrap(),
                primary_parameter: scaled_div_floor(primary_side, denominator),
                secondary_parameter: scaled_div_floor(secondary_side, -denominator),
            }))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum LineLineIntersection {
    Coincidental,
    Point(Intersection),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Intersection {
    side_signum: Signum,
    dir_signum: Signum,
    primary_parameter: i32,
    secondary_parameter: i32,
}

impl Intersection {
    /// Returns a number representing sing of the relative position of the
    /// secondary line point to the primary line.
    ///
    /// * [`Signum::Zero`] if the secondary line point lies on the primary
    ///   line.
    ///
    /// * [`Signum::Negative`] if secondary line point lies on the right side
    ///   from the primary line.
    ///
    /// * [`Signum::Positive`] if secondary line point on the on the left side
    ///   from the primary line.
    pub(super) fn side_signum(&self) -> Signum {
        self.side_signum
    }

    /// Returns a number representing sing of the secondary line direction
    /// relative to the primary line direction.
    ///
    /// * [`Signum::Zero`] if primary and secondary line directions are
    ///   coincidental or opposite (up to the scale).
    ///
    /// * [`Signum::Negative`] if secondary line direction points to the right
    ///   side from the primary line.
    ///
    /// * [`Signum::Positive`] if secondary line direction points to the left
    ///   side from the primary line.
    pub(super) fn dir_signum(&self) -> Signum {
        self.dir_signum
    }

    /// Returns the (fixed-point scaled) parameter of the line corresponding to
    /// the intersection point. Id est primary direction multiplied by the
    /// parameter (and re-scaled) corresponds to the difference between
    /// intersection point and the primary point.
    pub(super) fn primary_parameter(&self) -> i32 {
        self.primary_parameter
    }

    /// See [`Self::primary_parameter`].
    pub(super) fn secondary_parameter(&self) -> i32 {
        self.secondary_parameter
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Signum {
    Positive,
    Zero,
    Negative,
}

impl TryFrom<i32> for Signum {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value == 1 {
            Ok(Signum::Positive)
        } else if value == -1 {
            Ok(Signum::Negative)
        } else if value == 0 {
            Ok(Signum::Zero)
        } else {
            Err("Signum only accepts `1`, `0` and `-1`.")
        }
    }
}

impl Mul<i32> for Signum {
    type Output = i32;

    fn mul(self, rhs: i32) -> i32 {
        match self {
            Signum::Positive => rhs,
            Signum::Negative => -rhs,
            Signum::Zero => 0,
        }
    }
}

impl Mul<Signum> for Signum {
    type Output = Signum;

    fn mul(self, rhs: Signum) -> Signum {
        if self == Signum::Zero || rhs == Signum::Zero {
            Signum::Zero
        } else if self == rhs {
            Signum::Positive
        } else {
            Signum::Negative
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        let line_a = Line::new(IVec2::new(2048, 0), IVec2::new(724, -724));
        // coincidental
        let line_b = Line::new(IVec2::new(0, 2048), IVec2::new(-724, 724));
        // parallel
        let line_c = Line::new(IVec2::new(0, 0), IVec2::new(-724, 724));
        // crossing
        let line_d = Line::new(IVec2::new(1024, -4096), IVec2::new(0, 1024));
        let line_e = Line::new(IVec2::new(4096, 4096), IVec2::new(0, -1024));

        assert_eq!(
            line_a.intersection(line_b).unwrap(),
            LineLineIntersection::Coincidental
        );
        assert!(line_a.intersection(line_c).is_none());
        assert_eq!(
            line_a.intersection(line_d).unwrap(),
            LineLineIntersection::Point(Intersection {
                side_signum: Signum::Negative,
                dir_signum: Signum::Positive,
                primary_parameter: -1449,
                secondary_parameter: 5120,
            })
        );

        assert_eq!(
            line_a.intersection(line_e).unwrap(),
            LineLineIntersection::Point(Intersection {
                side_signum: Signum::Positive,
                dir_signum: Signum::Negative,
                primary_parameter: 2896,
                secondary_parameter: 6144,
            })
        );
    }

    #[test]
    fn test_projection() {
        let line = Line::new(IVec2::new(0, 2048), IVec2::new(724, -724));

        let negative = IVec2::new(line.point().x - 3072, line.point().y + 3072);
        let middle = IVec2::new(line.point().x + 3072, line.point().y - 3072);
        let right = IVec2::new(middle.x + 1024, middle.y + 1024);
        let left = IVec2::new(middle.x - 1024, middle.y - 1024);

        // assert_eq!(line.projection(negative), -4448256);
        // assert_eq!(line.projection(middle), 4448256);
        // assert_eq!(line.projection(left), 4448256);
        // assert_eq!(line.projection(right), 4448256);

        assert_eq!(line.projection(negative), -4344);
        assert_eq!(line.projection(middle), 4344);
        assert_eq!(line.projection(left), 4344);
        assert_eq!(line.projection(right), 4344);
    }

    #[test]
    fn test_side() {
        let line = Line::new(IVec2::new(0, 2048), IVec2::new(724, -724));
        let middle = IVec2::new(line.point().x + 3072, line.point().y - 3072);
        let right = IVec2::new(middle.x - 1024, middle.y - 1024);
        let left = IVec2::new(middle.x + 1024, middle.y + 1024);

        assert_eq!(line.side(middle), 0);
        assert_eq!(line.side(right), -1482752);
        assert_eq!(line.side(left), 1482752);

        assert_eq!(line.side_signum(middle), Signum::Zero);
        assert_eq!(line.side_signum(right), Signum::Negative);
        assert_eq!(line.side_signum(left), Signum::Positive);
    }
}
