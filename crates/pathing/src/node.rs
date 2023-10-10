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
pub(super) struct Node {
    prefix: Rc<PointChain>,
    interval: Interval,
    triangle_id: u32,
    min_distance: f32,
    /// Lower bound of the path length from the root via the interval the
    /// target.
    heuristic: f32,
}

impl Node {
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
            interval: Interval::Segment(interval),
            triangle_id,
            min_distance,
            heuristic,
        }
    }

    pub(super) fn root(&self) -> Point<f32> {
        self.prefix.point()
    }

    pub(super) fn edge_id(&self) -> Option<u32> {
        match self.interval {
            Interval::Target => None,
            Interval::Segment(ref interval) => Some(interval.edge_id()),
        }
    }

    pub(crate) fn triangle_id(&self) -> u32 {
        self.triangle_id
    }

    /// Returns distance of the node's interval and the target point.
    pub(super) fn min_distance(&self) -> f32 {
        self.min_distance
    }

    pub(super) fn expand_to_edge(
        &self,
        segment: Segment,
        step: Step,
        target: Point<f32>,
    ) -> [Option<Node>; 3] {
        let Interval::Segment(ref interval) = self.interval else {
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
            Some(Node::from_segment_interval(
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

    fn corner(
        &self,
        step: Step,
        segment: Segment,
        corner: Point<f32>,
        projection: ParamPair,
        target: Point<f32>,
    ) -> Node {
        let interval = SegmentInterval::from_projection(segment, projection, step.edge_id());
        let prefix = if self.root() == corner {
            Rc::clone(&self.prefix)
        } else {
            Rc::new(PointChain::extended(&self.prefix, corner))
        };

        Node::from_segment_interval(prefix, interval, step.triangle_id(), target)
    }

    pub(super) fn expand_to_target(&self, target: Point<f32>, triangle_id: u32) -> Option<Self> {
        let Interval::Segment(ref interval) = self.interval else {
            panic!("Cannot expand point interval.")
        };

        let prefix = match interval.cross(self.root(), target) {
            SegmentCross::Corner(point) => Rc::new(PointChain::extended(&self.prefix, point)),
            _ => Rc::clone(&self.prefix),
        };
        let heuristic = (target - prefix.point()).magnitude();
        Some(Self {
            prefix,
            interval: Interval::Target,
            triangle_id,
            min_distance: 0.,
            heuristic,
        })
    }

    pub(super) fn close(self, target: Point<f32>) -> Path {
        let chain = match self.interval {
            Interval::Target => PointChain::extended(&self.prefix, target),
            Interval::Segment(ref interval) => {
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

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.score() == other.score() && self.prefix.point() == other.prefix.point()
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score().partial_cmp(&self.score()).unwrap()
    }
}

#[derive(Clone)]
enum Interval {
    Target,
    Segment(SegmentInterval),
}
