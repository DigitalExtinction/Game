use glam::IVec2;

use super::{
    bounds::Bounds,
    line::{Line, LineLineIntersection, Signum},
};

#[derive(Clone)]
pub(super) struct Edge {
    line: Line,
    max: i32,
    signum: Signum,
    bounds: Option<Bounds>,
}

// Unfortunately, this must be implemented due to TinyVec trait bounds.
impl Default for Edge {
    fn default() -> Self {
        Self {
            line: Line::new(IVec2::ZERO, IVec2::X),
            max: 0,
            signum: Signum::Positive,
            bounds: None,
        }
    }
}

impl Edge {
    /// Creates a new edge boundary of a region.
    ///
    /// # Arguments
    ///
    /// * `line` - the edge is coincidental with this line and starts at the
    ///   `point` of `line`. `point` is included to the edge.
    ///
    /// * `signum` - if [`Signum::Positive`], the region entirely lies on the
    ///   left from `line`. If equal to [`Signum::Negative`], the region
    ///   entirely lies on the right from `line`. Remaining values are not
    ///   valid.
    ///
    /// * `max` - end of the edge is at `line` point plus `line` direction
    ///   scaled by this argument. It is exclusive.
    ///
    /// * `bounds` - parameter bounds of the underlying line given by the
    ///   maximum velocity circle. This is None if the line lies completely
    ///   outside of the maximum velocity circle.
    ///
    ///   Note that edge endpoints (corresponding to parameters 0 and `max`)
    ///   are independent on bounds.
    ///
    /// # Panics
    ///
    /// May panic if `signum` is [`Signum::Zero`].
    pub(super) fn new(line: Line, signum: Signum, max: i32, bounds: Option<Bounds>) -> Self {
        debug_assert!(signum != Signum::Zero);
        Self {
            line,
            signum,
            max,
            bounds,
        }
    }

    pub(super) fn line(&self) -> Line {
        self.line
    }

    /// Returns scaled parameter of the edge end point.
    pub(super) fn max(&self) -> i32 {
        self.max
    }

    pub(super) fn bounds(&self) -> Option<Bounds> {
        self.bounds
    }

    /// Computes and returns intersection between `self` and `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - the returned intersection parameters correspond to this
    ///   edge.
    ///
    /// * `inside` - true if `point` of the underlying line of `other` is
    ///   inside the region whose part `self` is.
    pub(super) fn intersection(&self, other: &Edge, inside: bool) -> Option<EdgeIntersection> {
        let intersection = match self.line.intersection(other.line) {
            Some(intersection) => intersection,
            None => return None,
        };

        match intersection {
            LineLineIntersection::Coincidental => {
                if inside {
                    // The intersections are used in generation of candidate
                    // velocities (points outside of all regions).
                    //
                    // It is necessary to avoid the situation of both of the
                    // coincidental lines being "fully inside" the other region
                    // because that might lead to omission of the optimal
                    // velocity.
                    //
                    // The opposite situation, i.e. yielding unnecessary (but
                    // acceptable) intersections, leads only to a small
                    // computational susceptibility.
                    Some(EdgeIntersection {
                        dir: IntersectionDir::InOut,
                        parameter: 0,
                    })
                } else {
                    None
                }
            }
            LineLineIntersection::Point(intersection) => {
                let primary_parameter = intersection.primary_parameter();
                if primary_parameter < 0 || primary_parameter >= self.max {
                    return None;
                }
                let secondary_parameter = intersection.secondary_parameter();
                if secondary_parameter < 0 || secondary_parameter >= other.max {
                    return None;
                }

                let dir = match self.signum * intersection.side_signum() {
                    Signum::Positive => IntersectionDir::InOut,
                    Signum::Negative => IntersectionDir::OutIn,
                    Signum::Zero => match self.signum * intersection.dir_signum() {
                        Signum::Positive => return None,
                        Signum::Negative => IntersectionDir::InOut,
                        Signum::Zero => unreachable!(),
                    },
                };

                Some(EdgeIntersection {
                    dir,
                    parameter: secondary_parameter,
                })
            }
        }
    }

    /// Returns true if `point` lies on the "inner" half-plane given by the
    /// line of `self`. The half-plane includes the line.
    pub(super) fn inner_side(&self, point: IVec2) -> bool {
        self.signum * self.line.side_signum(point) != Signum::Negative
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct EdgeIntersection {
    dir: IntersectionDir,
    parameter: i32,
}

impl EdgeIntersection {
    pub(super) fn dir(&self) -> IntersectionDir {
        self.dir
    }

    pub(super) fn parameter(&self) -> i32 {
        self.parameter
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IntersectionDir {
    OutIn,
    InOut,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        let right = IVec2::new(1024, 0);
        let left = IVec2::new(-1024, 0);
        let up = IVec2::new(0, 1024);

        let edge = Edge::new(
            Line::new(IVec2::ZERO, up),
            Signum::Positive,
            i32::MAX,
            Default::default(),
        );

        // point on `edge`
        assert!(edge
            .intersection(
                &Edge::new(
                    Line::new(IVec2::ZERO, left),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .is_none());
        assert_eq!(
            edge.intersection(
                &Edge::new(
                    Line::new(IVec2::ZERO, right),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .unwrap(),
            EdgeIntersection {
                dir: IntersectionDir::InOut,
                parameter: 0
            }
        );

        // negative primary parameter
        assert!(edge
            .intersection(
                &Edge::new(
                    Line::new(IVec2::new(1024, -1024), left),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .is_none());
        // negative secondary parameter
        assert!(edge
            .intersection(
                &Edge::new(
                    Line::new(IVec2::new(-1024, 1024), left),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .is_none());

        assert_eq!(
            edge.intersection(
                &Edge::new(
                    Line::new(IVec2::new(1024, 1024), left),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .unwrap(),
            EdgeIntersection {
                dir: IntersectionDir::OutIn,
                parameter: 1024
            }
        );

        assert_eq!(
            edge.intersection(
                &Edge::new(
                    Line::new(IVec2::new(-2048, 4096), right),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .unwrap(),
            EdgeIntersection {
                dir: IntersectionDir::InOut,
                parameter: 2048
            }
        );

        // coincidental
        assert_eq!(
            edge.intersection(
                &Edge::new(
                    Line::new(IVec2::new(0, 2048), up),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                true
            )
            .unwrap(),
            EdgeIntersection {
                dir: IntersectionDir::InOut,
                parameter: 0
            }
        );
        assert!(edge
            .intersection(
                &Edge::new(
                    Line::new(IVec2::new(2048, 0), up),
                    Signum::Positive,
                    i32::MAX,
                    Default::default()
                ),
                false
            )
            .is_none());
    }

    #[test]
    fn test_inner_side() {
        let edge_a = Edge::new(
            Line::new(IVec2::new(2048, 1024), IVec2::new(0, 1024)),
            Signum::Positive,
            12345,
            Default::default(),
        );
        let edge_b = Edge::new(
            Line::new(IVec2::new(2048, 1024), IVec2::new(0, 1024)),
            Signum::Negative,
            12345,
            Default::default(),
        );

        assert!(!edge_a.inner_side(IVec2::new(2049, 10000)));
        assert!(edge_b.inner_side(IVec2::new(2049, 10000)));
        assert!(edge_a.inner_side(IVec2::new(2047, 10000)));
        assert!(!edge_b.inner_side(IVec2::new(2047, 10000)));

        assert!(edge_a.inner_side(IVec2::new(2047, -10000)));
        assert!(edge_b.inner_side(IVec2::new(2049, -10000)));

        assert!(edge_a.inner_side(IVec2::new(2048, 10000)));
        assert!(edge_b.inner_side(IVec2::new(2048, 10000)));
    }
}
