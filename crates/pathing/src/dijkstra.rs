//! This module contains visibility graph based path finding algorithm.

use std::{cmp::Ordering, collections::BinaryHeap};

use ahash::AHashSet;
use bevy::core::FloatOrd;
use parry2d::{math::Point, query::PointQuery, shape::Segment};

use crate::{
    funnel::Funnel,
    geometry::{orient, which_side, Side},
    graph::VisibilityGraph,
    path::Path,
};

/// Finds and returns a reasonable path between two points.
///
/// Source and target points must not lie inside or on the edge of the same
/// triangle of the triangulation from which `graph` was created.
pub(crate) fn find_path(
    graph: &VisibilityGraph,
    source: PointContext,
    target: PointContext,
) -> Option<Path> {
    let mut open_set = OpenSet::new();
    let mut explored = AHashSet::new();

    let funnel = Funnel::new(source.point());
    for &edge_id in source.neighbours() {
        open_set.push(Step::from_segment(
            source.point(),
            &funnel,
            graph.geometry(edge_id).segment(),
            edge_id,
        ));
    }

    while let Some(step) = open_set.pop() {
        if !explored.insert(step.edge_id()) {
            continue;
        }

        let geometry = graph.geometry(step.edge_id());
        let segment = geometry.segment();

        for &next_edge_id in graph.neighbours(step.edge_id()) {
            if explored.contains(&next_edge_id) {
                continue;
            }

            let next_geom = graph.geometry(next_edge_id);
            if step.side() == which_side(segment.a, segment.b, next_geom.midpoint()) {
                continue;
            }

            open_set.push(Step::from_segment(
                geometry.midpoint(),
                step.funnel(),
                next_geom.segment(),
                next_edge_id,
            ));
        }

        if target.has_neighbour(step.edge_id()) {
            if step.side() == which_side(segment.a, segment.b, target.point()) {
                continue;
            }

            return Some(step.funnel().closed(target.point()));
        }
    }
    None
}

pub(crate) struct PointContext {
    point: Point<f32>,
    neighbours: Vec<u32>,
}

impl PointContext {
    /// Creates a new point context.
    ///
    /// # Arguments
    ///
    /// * `point` - position of the point in the map
    ///
    /// * `neighbours` - edge IDs of all neighboring edges. If the point lies
    ///   on en edge or its end points, the edge should not be included in the
    ///   vector.
    pub(crate) fn new(point: Point<f32>, neighbours: Vec<u32>) -> Self {
        Self { point, neighbours }
    }

    fn point(&self) -> Point<f32> {
        self.point
    }

    fn neighbours(&self) -> &[u32] {
        self.neighbours.as_slice()
    }

    fn has_neighbour(&self, edge_id: u32) -> bool {
        self.neighbours.contains(&edge_id)
    }
}

/// A priority queue of path exploration expansion steps.
struct OpenSet {
    heap: BinaryHeap<Step>,
}

impl OpenSet {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    fn pop(&mut self) -> Option<Step> {
        self.heap.pop()
    }

    fn push(&mut self, step: Step) {
        self.heap.push(step);
    }
}

/// A path exploration step -- a jump between two neighboring triangle edges /
/// line segments -- used in the edge/triangle graph traversal algorithm.
struct Step {
    score: FloatOrd,
    /// From which side the edge was approached. This is the side from the
    /// perspective of the edge's line segment before orientation.
    side: Side,
    /// Funnel expanded by all traversed edges from the starting point.
    /// `edge_id` is the first edge not used in the funnel expansion.
    funnel: Funnel,
    /// Edge to be traversed.
    edge_id: u32,
}

impl Step {
    fn from_segment(eye: Point<f32>, funnel: &Funnel, segment: Segment, edge_id: u32) -> Self {
        let side = which_side(segment.a, segment.b, eye);
        let segment = orient(eye, segment);
        let funnel = funnel.extended(segment);
        let dist = segment.distance_to_local_point(&funnel.tail().point(), true);
        Self::new(funnel.tail().length() + dist, side, funnel, edge_id)
    }

    fn new(score: f32, side: Side, funnel: Funnel, edge_id: u32) -> Self {
        Self {
            score: FloatOrd(score),
            side,
            funnel,
            edge_id,
        }
    }

    fn side(&self) -> Side {
        self.side
    }

    fn funnel(&self) -> &Funnel {
        &self.funnel
    }

    fn edge_id(&self) -> u32 {
        self.edge_id
    }
}

impl PartialEq for Step {
    fn eq(&self, other: &Step) -> bool {
        self.edge_id == other.edge_id && self.score == other.score
    }
}

impl Eq for Step {}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.score, other.edge_id).cmp(&(self.score, self.edge_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_set() {
        let mut set = OpenSet::new();
        set.push(Step::new(2., Side::Left, Funnel::new(Point::origin()), 1));
        set.push(Step::new(1.1, Side::Left, Funnel::new(Point::origin()), 2));
        set.push(Step::new(4., Side::Left, Funnel::new(Point::origin()), 3));
        assert_eq!(set.pop().unwrap().edge_id(), 2);
        assert_eq!(set.pop().unwrap().edge_id(), 1);
        assert_eq!(set.pop().unwrap().edge_id(), 3);
    }

    #[test]
    fn test_step_ord() {
        let step_a = Step::new(2., Side::Left, Funnel::new(Point::origin()), 1);
        let step_b = Step::new(2.1, Side::Left, Funnel::new(Point::origin()), 2);
        assert!(step_b < step_a);
    }
}
