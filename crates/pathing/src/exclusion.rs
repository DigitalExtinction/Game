use bevy::prelude::Transform;
use de_objects::Ichnography;
use de_types::projection::ToFlat;
use glam::EulerRot;
use parry2d::{
    math::{Isometry, Point},
    na, query,
    query::PointQuery,
    shape::ConvexPolygon,
};
use rstar::{Envelope, PointDistance, RTree, RTreeObject, SelectionFunction, AABB as RstarAABB};

/// Non accessible area on the map.
///
/// An area is considered inaccessible if a centroid of a moving object cannot
/// move inside. A padding / offset around static objects is included in the
/// area to accommodate for non-zero moving object sizes and moving object
/// trajectory smoothing.
#[derive(Clone, Debug)]
pub struct ExclusionArea {
    polygon: ConvexPolygon,
    aabb: RstarAABB<[f32; 2]>,
}

impl ExclusionArea {
    /// Collectively process exclusion areas and recursively merge overlapping
    /// areas.
    pub(crate) fn build(mut exclusions: Vec<Self>) -> Vec<Self> {
        let mut rtree: RTree<ExclusionArea> = RTree::new();

        for mut exclusion in exclusions.drain(..) {
            loop {
                let intersecting: Vec<ExclusionArea> =
                    rtree.drain_with_selection_function(&exclusion).collect();
                if intersecting.is_empty() {
                    rtree.insert(exclusion);
                    break;
                }
                exclusion = Self::merged(&exclusion, intersecting.as_slice());
            }
        }

        rtree.drain().collect()
    }

    /// Creates a new exclusion area from a static object ichnography and its
    /// world-to-object transform.
    pub fn from_ichnography(transform: &Transform, ichnography: &Ichnography) -> Self {
        let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
        let isometry = Isometry::new(transform.translation.to_flat().into(), angle);
        let vertices: Vec<Point<f32>> = ichnography
            .offset_convex_hull()
            .points()
            .iter()
            .map(|&p| isometry * p)
            .collect();

        Self::new(ConvexPolygon::from_convex_polyline(vertices).unwrap())
    }

    pub(crate) fn new(polygon: ConvexPolygon) -> Self {
        let aabb = polygon.local_aabb();
        Self {
            polygon,
            aabb: RstarAABB::from_corners([aabb.mins.x, aabb.mins.y], [aabb.maxs.x, aabb.maxs.y]),
        }
    }

    /// Returns a new exclusion area corresponding to the convex hull of 1 or
    /// more other exclusion areas.
    fn merged(primary: &ExclusionArea, exclusions: &[ExclusionArea]) -> Self {
        let points: Vec<Point<f32>> = exclusions
            .iter()
            .flat_map(|e| e.points())
            .chain(primary.points())
            .cloned()
            .collect();
        Self::new(ConvexPolygon::from_convex_hull(&points).unwrap())
    }

    /// Returns counter-clockwise points of the area's convex polygon.
    pub(crate) fn points(&self) -> &[Point<f32>] {
        self.polygon.points()
    }
}

impl RTreeObject for ExclusionArea {
    type Envelope = RstarAABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

impl PointDistance for ExclusionArea {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let point = Point::from_slice(point);
        let proj = self.polygon.project_local_point(&point, true);
        if proj.is_inside {
            0.
        } else {
            na::distance_squared(&point, &proj.point)
        }
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let point = Point::from_slice(point);
        self.polygon.contains_local_point(&point)
    }
}

impl SelectionFunction<ExclusionArea> for &ExclusionArea {
    fn should_unpack_parent(&self, envelope: &RstarAABB<[f32; 2]>) -> bool {
        self.aabb.intersects(envelope)
    }

    fn should_unpack_leaf(&self, other: &ExclusionArea) -> bool {
        query::intersection_test(
            &Isometry::identity(),
            &self.polygon,
            &Isometry::identity(),
            &other.polygon,
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let transform_a = Transform::from_xyz(0., 0., -1.);
        let ichnography_a = Ichnography::from(
            ConvexPolygon::from_convex_hull(&[
                Point::new(0., 3.),
                Point::new(0., -1.),
                Point::new(2., -1.),
                Point::new(2., 3.),
            ])
            .unwrap(),
        );
        let transform_b = Transform::default();
        let ichnography_b = Ichnography::from(
            ConvexPolygon::from_convex_hull(&[
                Point::new(0.5, 6.),
                Point::new(0.5, 5.),
                Point::new(1.5, 5.),
                Point::new(1.5, 6.),
            ])
            .unwrap(),
        );
        let transform_c = Transform::default();
        let ichnography_c = Ichnography::from(
            ConvexPolygon::from_convex_hull(&[
                Point::new(20., 20.),
                Point::new(20., 18.),
                Point::new(25., 18.),
                Point::new(25., 20.),
            ])
            .unwrap(),
        );

        let exclusions = ExclusionArea::build(vec![
            ExclusionArea::from_ichnography(&transform_a, &ichnography_a),
            ExclusionArea::from_ichnography(&transform_b, &ichnography_b),
            ExclusionArea::from_ichnography(&transform_c, &ichnography_c),
        ]);
        assert_eq!(exclusions.len(), 2);
        assert_eq!(
            exclusions[0].points(),
            &[
                Point::new(-2.0, 6.0),
                Point::new(-2.0, -2.0),
                Point::new(4.0, -2.0),
                Point::new(4.0, 6.0),
                Point::new(3.5, 8.0),
                Point::new(-1.5, 8.0),
            ]
        );
    }

    #[test]
    fn test_merged() {
        let a = ExclusionArea::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(-1., -1.),
                Point::new(-1., 1.),
                Point::new(1., 1.),
                Point::new(1., -1.),
            ])
            .unwrap(),
        );
        let b = ExclusionArea::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(-1.5, -1.5),
                Point::new(-1.5, 0.5),
                Point::new(0.5, 0.5),
                Point::new(0.5, -1.5),
            ])
            .unwrap(),
        );
        let area = ExclusionArea::merged(&a, &[b]);
        assert_eq!(
            area.points(),
            &[
                Point::new(1., 1.),
                Point::new(-1., 1.0),
                Point::new(-1.5, 0.5),
                Point::new(-1.5, -1.5),
                Point::new(0.5, -1.5),
                Point::new(1., -1.),
            ]
        );
    }
}
