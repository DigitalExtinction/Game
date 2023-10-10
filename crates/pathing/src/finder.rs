//! This module contains global map shortest path finder.

use ahash::AHashMap;
use bevy::prelude::{debug, info};
use de_map::size::MapBounds;
use de_types::path::Path;
use parry2d::{
    math::Point,
    na,
    query::PointQuery,
    shape::{Segment, Triangle},
};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use tinyvec::{ArrayVec, TinyVec};

use crate::{
    exclusion::ExclusionArea,
    graph::{Step, VisibilityGraph},
    polyanya::{find_path, PointContext},
    utils::HashableSegment,
    PathTarget,
};

/// A struct used for path finding.
pub struct PathFinder {
    /// Spatial index of triangles. It is used to find edges neighboring start
    /// and end pints of a path to be found.
    triangles: RTree<GraphTriangle>,
    /// All mutually exclusive exclusion areas which are not covered by
    /// `triangles`. It is used to find way out of unreachable area.
    exclusions: RTree<GraphExclusion>,
    graph: VisibilityGraph,
}

impl PathFinder {
    /// Creates a new path finder. It is assumed that there are no obstacles
    /// within `bounds`.
    pub(crate) fn new(bounds: &MapBounds) -> Self {
        let aabb = bounds.aabb();
        Self::from_triangles(
            vec![
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
            ],
            Vec::new(),
        )
    }

    /// Creates a new path finder based on a triangulated accessible area.
    ///
    /// # Arguments
    ///
    /// * `triangles` - the triangulation of the map. It must a) fully cover
    ///   area where objects (their centroids so there needs to be padding) can
    ///   freely move, b) contain not triangle-to-triangle intersections, c)
    ///   cover any of the area where object cannot freely move.
    ///
    /// * `exclusions` - mutually exclusive areas which fully cover area not
    ///   covered by `triangles`. There is no intersection between the
    ///   `exclusions` and `triangles`.
    pub(crate) fn from_triangles(
        mut triangles: Vec<Triangle>,
        mut exclusions: Vec<ExclusionArea>,
    ) -> Self {
        let mut graph = VisibilityGraph::new();

        let mut indexed_triangles = Vec::with_capacity(triangles.len());
        let mut segment_to_edge_id: AHashMap<HashableSegment, u32> =
            AHashMap::with_capacity(triangles.len() * 3);
        let mut tri_edge_ids = [0, 0, 0];

        for (triangle_id, triangle) in triangles.drain(..).enumerate() {
            let triangle_id: u32 = triangle_id.try_into().unwrap();

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
            indexed_triangles.push(GraphTriangle::new(triangle, triangle_id, tri_edge_ids));
            for [edge_id, neighbour_a, neighbour_b] in [
                [tri_edge_ids[0], tri_edge_ids[1], tri_edge_ids[2]],
                [tri_edge_ids[1], tri_edge_ids[2], tri_edge_ids[0]],
                [tri_edge_ids[2], tri_edge_ids[0], tri_edge_ids[1]],
            ] {
                graph.add_neighbours(edge_id, triangle_id, neighbour_a, neighbour_b);
            }
        }

        let exclusions: Vec<GraphExclusion> = exclusions
            .drain(..)
            .map(|exclusion| {
                let points = exclusion.points();
                debug_assert_ne!(points[0], points[points.len() - 1]);
                let edges = TinyVec::from_iter(
                    (0..points.len())
                        .map(|index| (points[index], points[(index + 1) % points.len()]))
                        .filter_map(|(a, b)| {
                            let hashable_segment = HashableSegment::new(Segment::new(a, b));
                            segment_to_edge_id.get(&hashable_segment).cloned()
                        }),
                );
                GraphExclusion::new(exclusion, edges)
            })
            .collect();

        debug!(
            "Creating path finder consisting of {} triangles and {} nodes",
            indexed_triangles.len(),
            graph.len(),
        );

        Self {
            triangles: RTree::bulk_load(indexed_triangles),
            exclusions: RTree::bulk_load(exclusions),
            graph,
        }
    }

    /// Returns a shortest path between two points.
    ///
    /// Returns `None` if there is no path between the two points.
    pub fn find_path<P: Into<Point<f32>>>(&self, from: P, target: PathTarget) -> Option<Path> {
        let from: Point<f32> = from.into();
        let to: Point<f32> = target.location().into();

        info!("Finding path from {:?} to {:?}", from, to);

        let source_edges = {
            let edges = self.locate_triangle_edges(from);
            if edges.is_empty() {
                self.locate_exclusion_edges(from)
            } else {
                edges
            }
        };
        if source_edges.is_empty() {
            return None;
        }

        let target_edges = self.locate_triangle_edges(to);
        if target_edges.is_empty() && target.properties().max_distance() == 0. {
            return None;
        }

        if source_edges.iter().any(|s| {
            target_edges
                .iter()
                .any(|t| s.triangle_id() == t.triangle_id())
        }) {
            debug!(
                "Trivial path from {:?} to {:?} found, trimming...",
                from, to
            );
            // Trivial case, both points are in the same triangle.
            return Path::straight(from.into(), target.location())
                .truncated(target.properties().distance());
        }

        let source = PointContext::new(from, source_edges);
        let target_context = PointContext::new(to, target_edges);
        match find_path(&self.graph, source, target_context, target.properties()) {
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

    fn locate_triangle_edges(&self, point: Point<f32>) -> Vec<Step> {
        let mut result = Vec::new();
        for triangle in self.triangles.locate_all_at_point(&[point.x, point.y]) {
            for edge_id in triangle.neighbours(point) {
                result.push(Step::new(edge_id, triangle.triangle_id()))
            }
        }
        result
    }

    fn locate_exclusion_edges(&self, point: Point<f32>) -> Vec<Step> {
        self.exclusions
            .locate_all_at_point(&[point.x, point.y])
            .flat_map(|t| {
                t.neighbours()
                    .iter()
                    .map(|&edge_id| Step::new(edge_id, u32::MAX))
            })
            .collect()
    }
}

/// A triangle used for spatial indexing inside the edge visibility graph.
struct GraphTriangle {
    triangle: Triangle,
    triangle_id: u32,
    /// IDs of edges of the triangle. These correspond to edges AB, BC and CA
    /// respectively.
    edges: [u32; 3],
}

impl GraphTriangle {
    fn new(triangle: Triangle, triangle_id: u32, edges: [u32; 3]) -> Self {
        Self {
            triangle,
            triangle_id,
            edges,
        }
    }

    fn triangle_id(&self) -> u32 {
        self.triangle_id
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

struct GraphExclusion {
    area: ExclusionArea,
    edges: TinyVec<[u32; 6]>,
}

impl GraphExclusion {
    fn new(area: ExclusionArea, edges: TinyVec<[u32; 6]>) -> Self {
        Self { area, edges }
    }

    fn neighbours(&self) -> &[u32] {
        self.edges.as_slice()
    }
}

impl RTreeObject for GraphExclusion {
    type Envelope = <ExclusionArea as RTreeObject>::Envelope;

    fn envelope(&self) -> Self::Envelope {
        self.area.envelope()
    }
}

impl PointDistance for GraphExclusion {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        self.area.distance_2(point)
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        self.area.contains_point(point)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use ntest::timeout;

    use super::*;
    use crate::PathQueryProps;

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
        let finder = PathFinder::from_triangles(triangles, vec![]);

        let first_path = finder
            .find_path(
                Vec2::new(-460., -950.),
                PathTarget::new(Vec2::new(450., 950.), PathQueryProps::exact(), false),
            )
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
            .find_path(
                Vec2::new(0.2, -950.),
                PathTarget::new(Vec2::new(0., 950.), PathQueryProps::exact(), false),
            )
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

        let third_path = finder
            .find_path(
                Vec2::new(0.2, -950.),
                PathTarget::new(
                    Vec2::new(0., 950.),
                    PathQueryProps::new(30., f32::INFINITY),
                    false,
                ),
            )
            .unwrap();
        assert_eq!(
            third_path.waypoints(),
            &[
                Vec2::new(0.59897804, 920.006),
                Vec2::new(18.6, 18.6),
                Vec2::new(18.6, -18.6),
                Vec2::new(0.2, -950.),
            ]
        );

        let forth_path = finder
            .find_path(
                Vec2::new(0.2, -950.),
                PathTarget::new(
                    Vec2::new(0., 950.),
                    PathQueryProps::new(999., f32::INFINITY),
                    false,
                ),
            )
            .unwrap();

        assert_eq!(
            forth_path.waypoints(),
            &[Vec2::new(18.003227, -48.80841), Vec2::new(0.2, -950.),]
        );

        let fifth_path = finder
            .find_path(
                Vec2::new(0.2, -950.),
                PathTarget::new(
                    Vec2::new(1., 8.),
                    PathQueryProps::new(0., f32::INFINITY),
                    false,
                ),
            )
            .unwrap();
        assert_eq!(
            fifth_path.waypoints(),
            &[
                Vec2::new(1.0000019, 18.6),
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
            Triangle::new(Point::new(0., 0.), Point::new(1., 1.), Point::new(1., 0.)),
            Triangle::new(Point::new(0., 0.), Point::new(0., 1.), Point::new(1., 1.)),
            Triangle::new(Point::new(0., 2.), Point::new(1., 3.), Point::new(1., 2.)),
            Triangle::new(Point::new(0., 2.), Point::new(0., 3.), Point::new(1., 3.)),
        ];

        let finder = PathFinder::from_triangles(triangles, vec![]);
        assert!(finder
            .find_path(
                Point::new(0.5, 2.5),
                PathTarget::new(Vec2::new(0.5, 0.5), PathQueryProps::exact(), false)
            )
            .is_none())
    }
}
