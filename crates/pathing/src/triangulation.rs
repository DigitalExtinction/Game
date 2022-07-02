//! This module implements Constrained Delaunay triangulation (CDT) based
//! triangulation of the accessible areas on the game map.

use ahash::AHashMap;
use bevy::core::FloatOrd;
use de_map::size::MapBounds;
use de_objects::EXCLUSION_OFFSET;
use parry2d::{
    math::{Point, Vector},
    shape::Triangle,
};
use spade::{handles::FixedVertexHandle, ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::exclusion::ExclusionArea;

const MAP_OFFSET: Vector<f32> = Vector::new(EXCLUSION_OFFSET, EXCLUSION_OFFSET);

/// Returns a triangulation of rectangular area given by `bounds` shrinked by
/// some distance with exclusion zones.
///
/// The returned triangles:
///
///  * do not intersect each other,
///  * cover the rectangle given by `bounds` except areas in `exclusions`,
///  * do not intersect any exclusion area given by `exclusions`.
///
/// # Arguments:
///
/// * `bounds` - area to be triangulated. This area is first shrinked by
///   [`de_objects::EXCLUSION_OFFSET`].
///
/// * `exclusions` - areas not to be included in the triangulation. Individual
///   exclusion areas must not intersect each other, must not touch map
///   boundaries and must be fully inside map boundaries.
///
/// # Panics
///
/// May panic if any of the aforementioned assumptions does not hold.
pub(crate) fn triangulate(bounds: &MapBounds, exclusions: &[ExclusionArea]) -> Vec<Triangle> {
    let mut triangulation = MapTriangulation::new();
    let (mins, maxs) = {
        let aabb = bounds.aabb();
        (aabb.mins + MAP_OFFSET, aabb.maxs - MAP_OFFSET)
    };
    triangulation.insert(Point::new(mins.x, mins.y), None);
    triangulation.insert(Point::new(mins.x, maxs.y), None);
    triangulation.insert(Point::new(maxs.x, maxs.y), None);
    triangulation.insert(Point::new(maxs.x, mins.y), None);

    for edge in MultipleAreaEdges::new(exclusions) {
        triangulation.insert(edge.a(), Some(edge.polygon_id()));
    }
    for edge in MultipleAreaEdges::new(exclusions) {
        triangulation.add_constraint(edge.a(), edge.b());
    }
    triangulation.collect()
}

/// This struct holds a mapping from vertices to polygon IDs.
struct MapTriangulation {
    triangulation: ConstrainedDelaunayTriangulation<Point2<f32>>,
    point_to_vertex: AHashMap<(FloatOrd, FloatOrd), Vertex>,
}

impl MapTriangulation {
    fn new() -> Self {
        Self {
            triangulation: ConstrainedDelaunayTriangulation::<Point2<f32>>::new(),
            point_to_vertex: AHashMap::new(),
        }
    }

    fn insert(&mut self, point: Point<f32>, polygon_id: Option<usize>) {
        let point2 = Point2::new(point.x, point.y);
        let handle = self.triangulation.insert(point2).unwrap();
        let old = self
            .point_to_vertex
            .insert(Self::point_to_key(point), Vertex::new(polygon_id, handle));
        debug_assert!(old.is_none());
    }

    fn add_constraint(&mut self, a: Point<f32>, b: Point<f32>) {
        let a = self.point_to_vertex.get(&Self::point_to_key(a)).unwrap();
        let b = self.point_to_vertex.get(&Self::point_to_key(b)).unwrap();
        self.triangulation.add_constraint(a.handle(), b.handle());
    }

    fn collect(self) -> Vec<Triangle> {
        self.triangulation
            .inner_faces()
            .filter_map(|f| {
                let vertices = f.vertices().map(|v| {
                    let v = v.as_ref();
                    Point::new(v.x, v.y)
                });
                let triangle = Triangle::new(vertices[0], vertices[1], vertices[2]);
                if self.is_excluded(&triangle) {
                    None
                } else {
                    Some(triangle)
                }
            })
            .collect()
    }

    fn point_to_key(point: Point<f32>) -> (FloatOrd, FloatOrd) {
        (FloatOrd(point.x), FloatOrd(point.y))
    }

    /// Returns true if the triangle is contained in an exclusion area.
    fn is_excluded(&self, triangle: &Triangle) -> bool {
        // We are using these facts in the following test:
        //  * all exclusion areas are convex
        //  * no exclusion areas overlap
        //
        // Knowing the above, it can be shown that a triangle is inside an
        // exclusion area iff all its vertices are part of the same exclusion
        // area polygon.

        let vertices = triangle.vertices().map(|p| {
            self.point_to_vertex
                .get(&Self::point_to_key(p))
                .and_then(|v| v.polygon_id())
        });
        vertices[0].is_some() && vertices[0] == vertices[1] && vertices[1] == vertices[2]
    }
}

struct Vertex {
    polygon_id: Option<usize>,
    hanlde: FixedVertexHandle,
}

impl Vertex {
    fn new(polygon_id: Option<usize>, hanlde: FixedVertexHandle) -> Self {
        Self { polygon_id, hanlde }
    }

    fn polygon_id(&self) -> Option<usize> {
        self.polygon_id
    }

    fn handle(&self) -> FixedVertexHandle {
        self.hanlde
    }
}

/// Iterator over all edges of all exclusion areas from a given slice.
struct MultipleAreaEdges<'a> {
    exclusions: &'a [ExclusionArea],
    index: usize,
    current: Option<SingleAreaEdges<'a>>,
}

impl<'a> MultipleAreaEdges<'a> {
    fn new(exclusions: &'a [ExclusionArea]) -> Self {
        Self {
            exclusions,
            index: 0,
            current: None,
        }
    }
}

impl<'a> Iterator for MultipleAreaEdges<'a> {
    type Item = ExclusionEdge;

    fn next(&mut self) -> Option<ExclusionEdge> {
        match self.current.as_mut().and_then(|c| c.next()) {
            Some(edge) => Some(edge),
            None => match self.exclusions.get(self.index) {
                Some(exclusion) => {
                    self.current = Some(SingleAreaEdges::new(exclusion, self.index));
                    self.index += 1;
                    self.current.as_mut().unwrap().next()
                }
                None => None,
            },
        }
    }
}

/// Iterator over all edges of a single exclusion area.
struct SingleAreaEdges<'a> {
    polygon: &'a ExclusionArea,
    polygon_id: usize,
    index: usize,
}

impl<'a> SingleAreaEdges<'a> {
    fn new(polygon: &'a ExclusionArea, polygon_id: usize) -> Self {
        Self {
            polygon,
            polygon_id,
            index: 0,
        }
    }
}

impl<'a> Iterator for SingleAreaEdges<'a> {
    type Item = ExclusionEdge;

    fn next(&mut self) -> Option<ExclusionEdge> {
        let points = self.polygon.points();
        if self.index >= points.len() {
            return None;
        }

        let a = points[self.index];
        self.index += 1;
        let b = points[self.index.rem_euclid(points.len())];
        Some(ExclusionEdge::new(self.polygon_id, a, b))
    }
}

/// Edge of a polygon of an exclusion area.
struct ExclusionEdge {
    /// ID of the polygon this edge belongs to.
    polygon_id: usize,
    a: Point<f32>,
    b: Point<f32>,
}

impl ExclusionEdge {
    fn new(polygon_id: usize, a: Point<f32>, b: Point<f32>) -> Self {
        Self { polygon_id, a, b }
    }

    fn polygon_id(&self) -> usize {
        self.polygon_id
    }

    fn a(&self) -> Point<f32> {
        self.a
    }

    fn b(&self) -> Point<f32> {
        self.b
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use ahash::AHashSet;
    use glam::Vec2;
    use parry2d::shape::ConvexPolygon;

    use super::*;
    use crate::utils::HashableSegment;

    #[test]
    fn test_triangulation_empty() {
        let obstacles = Vec::new();
        // <- 2.5 to left, <- 4.5 upwards
        let triangles = triangulate(
            &MapBounds::new(Vec2::new(
                19. + 2. * EXCLUSION_OFFSET,
                13. + 2. * EXCLUSION_OFFSET,
            )),
            &obstacles,
        );
        assert_eq!(triangles.len(), 2);

        let a = triangles[0];
        let b = triangles[1];
        assert_eq!(
            a,
            Triangle::new(
                Point::new(9.5, 6.5),
                Point::new(-9.5, 6.5),
                Point::new(-9.5, -6.5),
            )
        );
        assert_eq!(
            b,
            Triangle::new(
                Point::new(9.5, -6.5),
                Point::new(9.5, 6.5),
                Point::new(-9.5, -6.5),
            )
        );
    }

    #[test]
    fn test_triangulation() {
        let obstacles = vec![ExclusionArea::new(
            ConvexPolygon::from_convex_polyline(vec![
                Point::new(-0.1, 1.1),
                Point::new(-0.1, 1.3),
                Point::new(1.0, 1.3),
                Point::new(1.0, 1.1),
            ])
            .unwrap(),
        )];

        // <- 2.5 to left, <- 4.5 upwards
        let triangles: AHashSet<HashableTriangle> = triangulate(
            &MapBounds::new(Vec2::new(
                19. + 2. * EXCLUSION_OFFSET,
                13. + 2. * EXCLUSION_OFFSET,
            )),
            &obstacles,
        )
        .iter()
        .map(HashableTriangle::new)
        .collect();
        let expected: AHashSet<HashableTriangle> = vec![
            Triangle::new(
                Point::new(-0.1, 1.1),
                Point::new(-9.5, 6.5),
                Point::new(-9.5, -6.5),
            ),
            Triangle::new(
                Point::new(-9.5, -6.5),
                Point::new(9.5, -6.5),
                Point::new(-0.1, 1.1),
            ),
            Triangle::new(
                Point::new(1.0, 1.1),
                Point::new(-0.1, 1.1),
                Point::new(9.5, -6.5),
            ),
            Triangle::new(
                Point::new(-0.1, 1.3),
                Point::new(9.5, 6.5),
                Point::new(-9.5, 6.5),
            ),
            Triangle::new(
                Point::new(-0.1, 1.3),
                Point::new(-9.5, 6.5),
                Point::new(-0.1, 1.1),
            ),
            Triangle::new(
                Point::new(9.5, 6.5),
                Point::new(-0.1, 1.3),
                Point::new(1.0, 1.3),
            ),
            Triangle::new(
                Point::new(9.5, -6.5),
                Point::new(9.5, 6.5),
                Point::new(1.0, 1.1),
            ),
            Triangle::new(
                Point::new(1.0, 1.3),
                Point::new(1.0, 1.1),
                Point::new(9.5, 6.5),
            ),
        ]
        .iter()
        .map(HashableTriangle::new)
        .collect();

        assert_eq!(triangles, expected);
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    struct HashableTriangle(HashableSegment, HashableSegment, HashableSegment);

    impl HashableTriangle {
        fn new(triangle: &Triangle) -> Self {
            let mut edges = triangle.edges().map(HashableSegment::new);
            edges.sort();
            HashableTriangle(edges[0], edges[1], edges[2])
        }
    }
}
