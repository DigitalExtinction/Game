//! This module contains visibility graph based path finding algorithm.

use std::collections::BinaryHeap;

use ahash::AHashMap;
use bevy::utils::FloatOrd;
use de_types::path::Path;
use parry2d::math::Point;

use crate::{
    graph::{Step, VisibilityGraph},
    node::SearchNode,
    PathQueryProps,
};

const MAX_SEARCH_STEPS: u32 = 10_000_000;
const MAX_OPEN_SET_SIZE: usize = 1_000_000;

/// Finds and returns a reasonable path between two points.
///
/// Source and target points must not lie inside or on the edge of the same
/// triangle of the triangulation from which `graph` was created.
///
/// The path finding algorithm is based on (a modified) Polyanya:
///
/// Cui, M., Harabor, D. D., Grastien, A., & Data61, C. (2017, August).
/// Compromise-free Pathfinding on a Navigation Mesh. In IJCAI (pp. 496-502).
pub(crate) fn find_path(
    graph: &VisibilityGraph,
    source: PointContext,
    target: PointContext,
    properties: PathQueryProps,
) -> Option<Path> {
    let mut open_set = BinaryHeap::new();
    let mut visited = Visited::new();

    for &step in source.neighbours() {
        open_set.push(SearchNode::initial(
            source.point(),
            target.point(),
            graph.segment(step.edge_id()),
            step,
        ));
    }

    let Some(mut best) = open_set.peek().cloned() else {
        return None;
    };

    let mut counter = 0;
    while let Some(node) = open_set.pop() {
        counter += 1;
        if counter > MAX_SEARCH_STEPS {
            panic!("Path finding error: reached over {MAX_SEARCH_STEPS} search steps.");
        }
        if open_set.len() > MAX_OPEN_SET_SIZE {
            panic!("Path finding error: exploration open set is larger than {MAX_OPEN_SET_SIZE} nodes.");
        }

        let Some(edge_id) = node.edge_id() else {
            best = node.clone();
            break;
        };

        let worse = visited.test_push(node.root(), node.root_score());
        if worse {
            continue;
        }

        if best.min_distance() > node.min_distance() {
            best = node.clone();
        }

        if let Some(target_step) = target
            .neighbours()
            .iter()
            .find(|step| step.edge_id() == edge_id)
        {
            if let Some(expansion) =
                node.expand_to_target(target.point(), target_step.triangle_id())
            {
                open_set.push(expansion);
            }
            continue;
        }

        for &step in graph.neighbours(edge_id) {
            if step.triangle_id() == node.triangle_id() {
                // Allow only path forward (not backward through the just
                // traversed triangle).
                continue;
            }

            let next_segment = graph.segment(step.edge_id());
            for expansion in node
                .expand_to_edge(next_segment, step, target.point())
                .into_iter()
                .flatten()
            {
                open_set.push(expansion);
            }
        }
    }

    let path = best.close(target.point());
    let dist_to_target = path.waypoints()[0].distance(target.point().into());
    if dist_to_target > properties.max_distance() {
        None
    } else if dist_to_target < properties.distance() {
        path.truncated(properties.distance() - dist_to_target)
    } else {
        Some(path)
    }
}

pub(super) struct PointContext {
    point: Point<f32>,
    neighbours: Vec<Step>,
}

impl PointContext {
    /// Creates a new point context.
    ///
    /// # Arguments
    ///
    /// * `point` - position of the point in the map
    ///
    /// * `neighbours` - steps to all neighboring edges. If the point lies
    ///   on an edge or its end points, the edge should not be included in the
    ///   vector.
    pub(super) fn new(point: Point<f32>, neighbours: Vec<Step>) -> Self {
        Self { point, neighbours }
    }

    fn point(&self) -> Point<f32> {
        self.point
    }

    fn neighbours(&self) -> &[Step] {
        self.neighbours.as_slice()
    }
}

struct Visited(AHashMap<(FloatOrd, FloatOrd), f32>);

impl Visited {
    fn new() -> Self {
        Self(AHashMap::new())
    }

    /// Marks a point as visited and stores/updates its associated score.
    ///
    /// Returns true when the point was already visited and the previous score
    /// was smaller than the new score.
    fn test_push(&mut self, point: Point<f32>, score: f32) -> bool {
        let key = (FloatOrd(point.x), FloatOrd(point.y));
        let current_score = self.0.get(&key).cloned().unwrap_or(f32::INFINITY);
        if current_score > score {
            self.0.insert(key, score);
            false
        } else {
            current_score != score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visited() {
        let mut visited = Visited::new();

        assert!(!visited.test_push(Point::new(1., 2.), 8.));
        assert!(!visited.test_push(Point::new(1., 2.), 7.));
        assert!(visited.test_push(Point::new(1., 2.), 7.5));

        assert!(!visited.test_push(Point::new(3., 2.), 11.));
        assert!(visited.test_push(Point::new(3., 2.), 12.));
        assert!(!visited.test_push(Point::new(3., 2.), 7.));
    }
}
