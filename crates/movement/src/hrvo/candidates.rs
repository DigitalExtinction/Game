use glam::IVec2;

use super::{
    edge::Edge,
    line::Line,
    parameters::{ParameterIterator, Transition},
    region::Region,
    scale::vec_div_to_scale,
};

/// An iterator over candidate feasible velocities. One of the yielded is
/// guaranteed to be optimal.
pub(super) struct Candidates<'a> {
    desired: IVec2,
    regions: &'a [Region],
    region_index: usize,
    edges: &'a [Edge],
    edge_index: usize,
    candidates: Option<EdgeCandidates>,
}

impl<'a> Candidates<'a> {
    pub(super) fn new(desired: IVec2, regions: &'a [Region]) -> Self {
        let edges = regions[0].edges();
        Self {
            desired,
            regions,
            region_index: 1,
            edges,
            edge_index: 0,
            candidates: None,
        }
    }
}

impl<'a> Iterator for Candidates<'a> {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(candidates) = &mut self.candidates {
                if let Some(candidate) = candidates.next() {
                    break Some(candidate);
                }
            }

            if self.edge_index >= self.edges.len() {
                if self.region_index >= self.regions.len() {
                    break None;
                }

                self.edge_index = 0;
                self.edges = self.regions[self.region_index].edges();
                self.region_index += 1;
            }

            let edge = &self.edges[self.edge_index];
            self.edge_index += 1;

            let index = self.region_index - 1;
            self.candidates = EdgeCandidates::compute(edge, self.desired, index, self.regions);
        }
    }
}

/// Iterator over feasible velocities on an edge. If the optimal velocity lies
/// on the edge, it is guaranteed to be yielded.
struct EdgeCandidates {
    line: Line,
    parameters: ParameterIterator,
}

impl EdgeCandidates {
    /// Computes and returns new edge candidate iterator. The function might
    /// return `None` if there are no feasible candidates.
    fn compute(
        edge: &Edge,
        desired: IVec2,
        primary_index: usize,
        regions: &[Region],
    ) -> Option<Self> {
        let bounds = match edge.bounds() {
            Some(bounds) => bounds,
            None => return None,
        };

        let min = bounds.min().max(0);
        let max = bounds.max().min(edge.max());
        if min > max {
            return None;
        }

        let mut num_inside = 0;
        let mut transitions: Vec<Transition> = vec![
            Transition::new(min, 0),
            Transition::new(edge.line().projection(desired), 0),
            Transition::new(max, 0),
        ];

        for i in 0..regions.len() {
            if i == primary_index {
                continue;
            }

            let region = &regions[i];

            let point_inside = region.contains(edge.line().point());
            if point_inside {
                num_inside += 1;
            }

            transitions.extend_from_slice(region.intersections(edge, point_inside).as_slice());
        }

        Some(Self {
            parameters: ParameterIterator::new(num_inside, bounds, transitions),
            line: edge.line(),
        })
    }
}

impl Iterator for EdgeCandidates {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.parameters.next().map(|parameter| {
            let delta = vec_div_to_scale(parameter * self.line.dir());
            self.line.point() + delta
        })
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use tinyvec::ArrayVec;

    use super::*;
    use crate::hrvo::{bounds::Bounds, line::Signum, scale::vec_from_scale};

    fn cone(apex: IVec2, left: IVec2, right: IVec2, radius: f32) -> Region {
        let mut edges = ArrayVec::new();

        edges.push(Edge::new(
            Line::new(apex, left),
            Signum::Negative,
            i32::MAX,
            Bounds::compute(vec_from_scale(apex), vec_from_scale(left), radius.powi(2)),
        ));
        edges.push(Edge::new(
            Line::new(apex, right),
            Signum::Positive,
            i32::MAX,
            Bounds::compute(vec_from_scale(apex), vec_from_scale(right), radius.powi(2)),
        ));
        Region::new(edges)
    }

    #[test]
    fn test_candidates_sinle_cone() {
        let desired = IVec2::new(0, 2048);
        let regions = vec![cone(
            IVec2::new(512, 1024),
            IVec2::new(-724, 724),
            IVec2::new(0, 1024),
            10.,
        )];

        let retrieved = AHashSet::from_iter(Candidates::new(desired, &regions));
        let mut expected = AHashSet::new();
        // the left edge hitting maximum velocity circle
        expected.insert(IVec2::new(-6432, 7967));
        // the right edge hitting maximum velocity circle
        expected.insert(IVec2::new(512, 10227));
        // projection of desired velocity on the left edge
        expected.insert(IVec2::new(-256, 1791));
        // projection of desired velocity on the right edge
        expected.insert(IVec2::new(512, 2048));
        // apex
        expected.insert(IVec2::new(512, 1024));
        assert_eq!(retrieved, expected,);
    }

    #[test]
    fn test_candidates_two_cones() {
        let desired = IVec2::new(0, 2048);
        let regions = vec![
            // Cone A
            cone(
                IVec2::new(512, 1024),
                IVec2::new(-724, 724),
                IVec2::new(0, 1024),
                10.,
            ),
            // Cone B
            cone(
                IVec2::new(-256, 2048),
                IVec2::new(724, 724),
                IVec2::new(724, -724),
                10.,
            ),
        ];

        let retrieved = AHashSet::from_iter(Candidates::new(desired, &regions));
        let mut expected = AHashSet::new();
        // Apex of Cone B, projection on right edge of Cone A and projection on
        // both edges of Cone B are inside the other cone and thus not present
        // in the yielded points.
        //
        // left edge of Cone A hitting maximum velocity circle
        expected.insert(IVec2::new(-6432, 7967));
        // right edge of Cone A hitting maximum velocity circle
        expected.insert(IVec2::new(512, 10227));
        // apex of cone A
        expected.insert(IVec2::new(512, 1024));
        // left edge of Cone B hitting maximum velocity circle
        expected.insert(IVec2::new(5996, 8300));
        // right edge of Cone B hitting maximum velocity circle
        expected.insert(IVec2::new(8079, -6288));
        // projection of desired velocity on left edge of Cone A
        expected.insert(IVec2::new(-256, 1791));
        // Intersection of left edge of Cone B and right edge of Cone A
        expected.insert(IVec2::new(512, 2816));
        expected.insert(IVec2::new(511, 2815));
        // Intersection of right edge of Cone B and right edge of Cone A
        expected.insert(IVec2::new(512, 1280));
        expected.insert(IVec2::new(511, 1280));

        assert_eq!(retrieved, expected);
    }

    #[test]
    fn test_edge_candidates() {
        let edge = Edge::new(
            // origin: (0.5, 1), direction: (1, 0)
            Line::new(IVec2::new(512, 1024), IVec2::new(1024, 0)),
            Signum::Positive,
            3584,                         // 3.5 (thus x = 4)
            Some(Bounds::new(256, 2048)), // 0.25 (thus x = 0.75) - 2.5 (thus x = 2.5)
        );
        let desired = IVec2::new(800, 639);

        // Skipped region (corresponding to `edge`)
        let region_zero = cone(
            IVec2::new(10000, 10000),
            IVec2::new(0, 1024),
            IVec2::new(1024, 0),
            20.,
        );

        // Region A
        //   apex:  (1.1, 0.0)
        //   left:  (0.0, 1.0)
        //   right: (0.7, 0.7)
        let region_a = cone(
            IVec2::new(1126, 0),
            IVec2::new(0, 1024),
            IVec2::new(724, 724),
            20.,
        );

        // Region B
        //   apex:  (1.2, 0.5)
        //   left:  (0.0, 1.0)
        //   right: (0.7, 0.7)
        let region_b = cone(
            IVec2::new(1228, 512),
            IVec2::new(0, 1024),
            IVec2::new(724, 724),
            20.,
        );

        // Region C
        //   apex:  (3.0, 0.5)
        //   left:  (0.0, 1.0)
        //   right: (1.0, 0.0)
        let region_c = cone(
            IVec2::new(3072, 512),
            IVec2::new(0, 1024),
            IVec2::new(1024, 0),
            20.,
        );

        let regions = vec![region_b, region_zero, region_a, region_c];

        // * Region B is fully inside region A => no intersections expected.
        // * Region C has intersections outside of `edge` bounds.
        // * Region A intersects at (1.1, 1.0) and (2.1, 1.0)
        let candidates: Vec<IVec2> = EdgeCandidates::compute(&edge, desired, 1, regions.as_slice())
            .unwrap()
            .collect();

        assert_eq!(
            candidates.as_slice(),
            &[
                IVec2::new(768, 1024),
                IVec2::new(800, 1024),
                IVec2::new(1126, 1024),
                IVec2::new(2150, 1024),
                IVec2::new(2560, 1024)
            ]
        );
    }
}
