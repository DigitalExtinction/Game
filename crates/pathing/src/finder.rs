//! This module contains global map shortest path finder.

use ahash::AHashMap;
use bevy::prelude::{debug, info};
use de_map::size::MapBounds;
use parry2d::{math::Point, na, query::PointQuery, shape::Triangle};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use tinyvec::ArrayVec;

use crate::{
    dijkstra::{find_path, PointContext},
    graph::VisibilityGraph,
    path::Path,
    utils::HashableSegment,
};

/// A struct used for path finding.
pub(crate) struct PathFinder {
    /// Spatial index of triangles. It is used to find edges neighboring start
    /// and end pints of a path to be found.
    triangles: RTree<GraphTriangle>,
    graph: VisibilityGraph,
}

impl PathFinder {
    /// Creates a new path finder. It is assumed that there are no obstacles
    /// within `bounds`.
    #[allow(dead_code)]
    pub(crate) fn new(bounds: &MapBounds) -> Self {
        let aabb = bounds.aabb();
        Self::from_triangles(vec![
            Triangle::new(
                Point::new(aabb.mins.x, aabb.maxs.y),
                Point::new(aabb.mins.x, aabb.mins.y),
                Point::new(aabb.maxs.x, aabb.mins.y),
            ),
            Triangle::new(
                Point::new(aabb.mins.x, aabb.maxs.y),
                Point::new(aabb.maxs.x, aabb.mins.y),
                Point::new(aabb.maxs.x, aabb.maxs.y),
            ),
        ])
    }

    /// Creates a new path finder based on a triangulated accessible area.
    ///
    /// # Arguments
    ///
    /// * `triangles` - the triangulation of the map. It must a) fully cover
    ///   area where objects (their centroids so there needs to be padding) can
    ///   freely move, b) contain not triangle-to-triangle intersections, c)
    ///   cover any of the area where object cannot freely move.
    #[allow(dead_code)]
    pub(crate) fn from_triangles(triangles: Vec<Triangle>) -> Self {
        let mut graph = VisibilityGraph::new();

        let mut indexed_triangles = Vec::with_capacity(triangles.len());
        let mut segment_to_edge_id: AHashMap<HashableSegment, u32> =
            AHashMap::with_capacity(triangles.len() * 3);
        let mut tri_edge_ids = [0, 0, 0];

        for triangle in triangles {
            let segments = triangle.edges();
            for i in 0..3 {
                let segment = segments[i];
                let hashable_segment = HashableSegment::new(segment);
                tri_edge_ids[i] = match segment_to_edge_id.get(&hashable_segment) {
                    Some(index) => *index,
                    None => {
                        let edge_id = graph.new_node(segment);
                        segment_to_edge_id.insert(hashable_segment, edge_id);
                        edge_id
                    }
                };
            }
            indexed_triangles.push(GraphTriangle::new(triangle, tri_edge_ids));
            for [edge_id, neighbour_a, neighbour_b] in [
                [tri_edge_ids[0], tri_edge_ids[1], tri_edge_ids[2]],
                [tri_edge_ids[1], tri_edge_ids[2], tri_edge_ids[0]],
                [tri_edge_ids[2], tri_edge_ids[0], tri_edge_ids[1]],
            ] {
                graph.add_neighbours(edge_id, neighbour_a, neighbour_b);
            }
        }

        debug!(
            "Creating path finder consisting of {} triangles and {} nodes",
            indexed_triangles.len(),
            graph.len(),
        );

        Self {
            triangles: RTree::bulk_load(indexed_triangles),
            graph,
        }
    }

    /// Returns a shortest path between two points.
    ///
    /// Returns `None` if there is no path between the two points.
    #[allow(dead_code)]
    pub(crate) fn find_path<P: Into<Point<f32>>>(&self, from: P, to: P) -> Option<Path> {
        let from: Point<f32> = from.into();
        let to: Point<f32> = to.into();

        info!("Finding path from {:?} to {:?}", from, to);

        let source_edges = self.locate_edges(from);
        if source_edges.is_empty() {
            return None;
        }
        let target_edges = self.locate_edges(to);
        if target_edges.is_empty() {
            return None;
        }

        if source_edges
            .iter()
            .filter(|s| target_edges.contains(s))
            .take(2)
            .count()
            >= 2
        {
            // Trivial case, both points are in the same triangle.
            debug!("Trivial path from {:?} to {:?} found", from, to);
            return Some(Path::straight(from, to));
        }

        let source = PointContext::new(from, source_edges);
        let target = PointContext::new(to, target_edges);
        match find_path(&self.graph, source, target) {
            Some(path) => {
                debug!(
                    "Path of length {} from {:?} to {:?} found",
                    path.length(),
                    from,
                    to
                );
                Some(path)
            }
            None => {
                debug!("No path from {:?} to {:?} found", from, to);
                None
            }
        }
    }

    fn locate_edges(&self, point: Point<f32>) -> Vec<u32> {
        self.triangles
            .locate_all_at_point(&[point.x, point.y])
            .flat_map(|t| t.neighbours(point))
            .collect()
    }
}

/// A triangle used for spatial indexing inside the edge visibility graph.
struct GraphTriangle {
    triangle: Triangle,
    /// IDs of edges of the triangle. These correspond to edges AB, BC and CA
    /// respectively.
    edges: [u32; 3],
}

impl GraphTriangle {
    fn new(triangle: Triangle, edges: [u32; 3]) -> Self {
        Self { triangle, edges }
    }

    /// Returns (up to 3) IDs of the triangle edges excluding edges which
    /// include `point`.
    fn neighbours(&self, point: Point<f32>) -> ArrayVec<[u32; 3]> {
        debug_assert!(self.triangle.contains_local_point(&point));

        let mut edge_ids = ArrayVec::new();
        for (i, edge) in self.triangle.edges().iter().enumerate() {
            if !edge.contains_local_point(&point) {
                edge_ids.push(self.edges[i]);
            }
        }
        edge_ids
    }
}

impl RTreeObject for GraphTriangle {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let aabb = self.triangle.local_aabb();
        AABB::from_corners([aabb.mins.x, aabb.mins.y], [aabb.maxs.x, aabb.maxs.y])
    }
}

impl PointDistance for GraphTriangle {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let point = Point::from_slice(point);
        let proj = self.triangle.project_local_point(&point, true);
        if proj.is_inside {
            0.
        } else {
            na::distance_squared(&point, &proj.point)
        }
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let point = Point::from_slice(point);
        self.triangle.contains_local_point(&point)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use ntest::timeout;

    use super::*;

    #[test]
    fn test_finder() {
        let triangles = vec![
            Triangle::new(
                Point::new(-18.6, -18.6),
                Point::new(-500., 1000.),
                Point::new(-500., -1000.),
            ),
            Triangle::new(
                Point::new(-18.6, -18.6),
                Point::new(-500., -1000.),
                Point::new(18.6, -18.6),
            ),
            Triangle::new(
                Point::new(500., -1000.),
                Point::new(18.6, -18.6),
                Point::new(-500., -1000.),
            ),
            Triangle::new(
                Point::new(500., 1000.),
                Point::new(-500., 1000.),
                Point::new(18.6, 18.6),
            ),
            Triangle::new(
                Point::new(-18.6, 18.6),
                Point::new(-500., 1000.),
                Point::new(-18.6, -18.6),
            ),
            Triangle::new(
                Point::new(-18.6, 18.6),
                Point::new(18.6, 18.6),
                Point::new(-500., 1000.),
            ),
            Triangle::new(
                Point::new(500., -1000.),
                Point::new(500., 1000.),
                Point::new(18.6, -18.6),
            ),
            Triangle::new(
                Point::new(18.6, 18.6),
                Point::new(18.6, -18.6),
                Point::new(500., 1000.),
            ),
        ];
        let finder = PathFinder::from_triangles(triangles);

        let first_path = finder
            .find_path(Vec2::new(-460., -950.), Vec2::new(450., 950.))
            .unwrap();
        assert_eq!(
            first_path.waypoints(),
            &[
                Vec2::new(450., 950.),
                Vec2::new(-18.6, 18.6),
                Vec2::new(-460., -950.),
            ]
        );

        let second_path = finder
            .find_path(Vec2::new(0.2, -950.), Vec2::new(0., 950.))
            .unwrap();
        assert_eq!(
            second_path.waypoints(),
            &[
                Vec2::new(0., 950.),
                Vec2::new(18.6, 18.6),
                Vec2::new(18.6, -18.6),
                Vec2::new(0.2, -950.),
            ]
        );
    }

    #[test]
    #[timeout(100)]
    fn test_unreachable() {
        let triangles = vec![
            Triangle::new(
                Point::new(0., 0.),
                Point::new(-1., 1.),
                Point::new(-1., -1.),
            ),
            Triangle::new(Point::new(0., 0.), Point::new(1., 1.), Point::new(-1., 1.)),
            Triangle::new(Point::new(0., 0.), Point::new(1., -1.), Point::new(1., 1.)),
            Triangle::new(
                Point::new(0., 0.),
                Point::new(-1., -1.),
                Point::new(1., -1.),
            ),
            Triangle::new(
                Point::new(0., 30.),
                Point::new(0., 20.),
                Point::new(10., 20.),
            ),
        ];

        let finder = PathFinder::from_triangles(triangles);
        assert!(finder
            .find_path(Point::new(-0.5, 0.), Point::new(2., 22.))
            .is_none())
    }
}
