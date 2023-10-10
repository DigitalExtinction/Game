use std::{cmp::Ordering, rc::Rc};

use de_types::path::Path;
use parry2d::{math::Point, shape::Segment};

use crate::{
    chain::PointChain,
    graph::Step,
    interval::{ParamPair, SegmentCross, SegmentInterval},
};

/// Polyanya search node.
///
/// The node consists of a path prefix (whose last point is root point of the
/// node), an interval (a segment or the target point) and search heuristic.
#[derive(Clone)]
pub(super) struct SearchNode {
    prefix: Rc<PointChain>,
    point_set: PointSet,
    triangle_id: u32,
    min_distance: f32,
    /// Lower bound of the path length from the root via the interval the
    /// target.
    heuristic: f32,
}

impl SearchNode {
    /// Creates an initial node, i.e. a node whose prefix consists of a single
    /// point: `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - starting point.
    ///
    /// * `target` - path finding target point.
    ///
    /// * `segment` - first segment to be traversed.
    ///
    /// * `step` - first point-to-edge step in the triangle edge neighboring
    ///   graph.
    pub(super) fn initial(
        source: Point<f32>,
        target: Point<f32>,
        segment: Segment,
        step: Step,
    ) -> Self {
        Self::from_segment_interval(
            Rc::new(PointChain::first(source)),
            SegmentInterval::new(segment, true, true, step.edge_id()),
            step.triangle_id(),
            target,
        )
    }

    /// Creates a new Polyanya node from a path prefix and an interval. Node
    /// heuristic is computed.
    ///
    /// # Arguments
    ///
    /// * `prefix` - node path prefix (up to the root of the node).
    ///
    /// * `interval` - part of a triangle edge corresponding the expansion of
    ///   this node. I.e. set (line segment) of "furthest" explored points
    ///   along this particular path expansion.
    ///
    /// * `triangle_id` - last traversed triangle (to reach `interval`).
    ///
    /// * `target` - searched path target.
    fn from_segment_interval(
        prefix: Rc<PointChain>,
        interval: SegmentInterval,
        triangle_id: u32,
        target: Point<f32>,
    ) -> Self {
        let cross = interval.cross(prefix.point(), target).point();
        let heuristic = (cross - prefix.point()).magnitude() + (target - cross).magnitude();
        let min_distance = interval.distance_to_point(target);

        Self {
            prefix,
            point_set: PointSet::Segment(interval),
            triangle_id,
            min_distance,
            heuristic,
        }
    }

    pub(super) fn root(&self) -> Point<f32> {
        self.prefix.point()
    }

    pub(super) fn edge_id(&self) -> Option<u32> {
        match self.point_set {
            PointSet::Target => None,
            PointSet::Segment(ref interval) => Some(interval.edge_id()),
        }
    }

    pub(crate) fn triangle_id(&self) -> u32 {
        self.triangle_id
    }

    /// Returns distance of the node's interval and the target point.
    pub(super) fn min_distance(&self) -> f32 {
        self.min_distance
    }

    /// Constructs and returns expansion of self onto a next (adjacent) edge.
    ///
    /// # Arguments
    ///
    /// * `segment` - full line segment of the next edge.
    ///
    /// * `step` - single triangle traversal step.
    ///
    /// * `target` - path searching target point.
    ///
    /// # Panics
    ///
    /// Panics if the last crossed triangle on the path to this node
    /// corresponds to the triangle of the next step (i.e. if the expansion
    /// goes backwards).
    pub(super) fn expand_to_edge(
        &self,
        segment: Segment,
        step: Step,
        target: Point<f32>,
    ) -> [Option<Self>; 3] {
        assert!(step.triangle_id() != self.triangle_id);

        let PointSet::Segment(ref interval) = self.point_set else {
            panic!("Cannot expand point interval.")
        };

        let projection = interval.project_onto_segment(self.prefix.point(), segment);

        let node_a = if let Some(a_corner) = interval.a_corner() {
            projection
                .side_a()
                .map(|projection| self.corner(step, segment, a_corner, projection, target))
        } else {
            None
        };

        let node_mid = if let Some(projection) = projection.middle() {
            let interval = SegmentInterval::from_projection(segment, projection, step.edge_id());
            Some(Self::from_segment_interval(
                Rc::clone(&self.prefix),
                interval,
                step.triangle_id(),
                target,
            ))
        } else {
            None
        };

        let node_b = if let Some(b_corner) = interval.b_corner() {
            projection
                .side_b()
                .map(|projection| self.corner(step, segment, b_corner, projection, target))
        } else {
            None
        };

        [node_a, node_mid, node_b]
    }

    /// Creates a new node whose prefix is equal to the prefix of self with
    /// potential addition of `corner` (as the new node root) in the case it
    /// differs from root of self.
    ///
    /// # Arguments
    ///
    /// * `step` - the one additional step from self to reach the to be created
    ///   node.
    ///
    /// * `segment` - line segment corresponding to the full edge directly
    ///   reached (i.e. via a single triangle) from self.
    ///
    /// * `corner` - last path bend / corner to reach `projection` onto
    ///   `segment`. Id est root of the to be created node.
    ///
    /// * `projection` - part of the target edge.
    ///
    /// * `target` - searched path target.
    fn corner(
        &self,
        step: Step,
        segment: Segment,
        corner: Point<f32>,
        projection: ParamPair,
        target: Point<f32>,
    ) -> Self {
        let interval = SegmentInterval::from_projection(segment, projection, step.edge_id());
        let prefix = if self.root() == corner {
            Rc::clone(&self.prefix)
        } else {
            Rc::new(PointChain::extended(&self.prefix, corner))
        };

        Self::from_segment_interval(prefix, interval, step.triangle_id(), target)
    }

    pub(super) fn expand_to_target(&self, target: Point<f32>, triangle_id: u32) -> Option<Self> {
        let PointSet::Segment(ref interval) = self.point_set else {
            panic!("Cannot expand point interval.")
        };

        let prefix = match interval.cross(self.root(), target) {
            SegmentCross::Corner(point) => Rc::new(PointChain::extended(&self.prefix, point)),
            _ => Rc::clone(&self.prefix),
        };
        let heuristic = (target - prefix.point()).magnitude();
        Some(Self {
            prefix,
            point_set: PointSet::Target,
            triangle_id,
            min_distance: 0.,
            heuristic,
        })
    }

    /// Constructs path from the search node.
    ///
    /// The resulting path is a full path from source to target if the node
    /// (self) corresponds to the target point. Otherwise, it corresponds to
    /// the path from source to the closest point to target in the point set of
    /// self (on the nodes line segment).
    pub(super) fn close(self, target: Point<f32>) -> Path {
        let chain = match self.point_set {
            PointSet::Target => PointChain::extended(&self.prefix, target),
            PointSet::Segment(ref interval) => {
                PointChain::extended(&self.prefix, interval.project_point(target))
            }
        };
        chain.to_path()
    }

    pub(super) fn root_score(&self) -> f32 {
        self.prefix.length()
    }

    fn score(&self) -> f32 {
        self.root_score() + self.heuristic
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score() && self.prefix.point() == other.prefix.point()
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score().partial_cmp(&self.score()).unwrap()
    }
}

#[derive(Clone)]
enum PointSet {
    /// Point set (of cardinality 1) corresponding to the path searching target
    /// point.
    Target,
    /// Point set corresponding to an interval (which is either a part or a
    /// full triangle edge).
    Segment(SegmentInterval),
}
