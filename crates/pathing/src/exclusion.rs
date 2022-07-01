use ahash::AHashMap;
use bevy::prelude::GlobalTransform;
use de_core::{objects::ObjectType, projection::ToFlat};
use de_objects::{Ichnography, IchnographyCache};
use glam::{EulerRot, IVec2};
use parry2d::{
    bounding_volume::{BoundingVolume, AABB},
    math::{Isometry, Point},
    query,
    shape::ConvexPolygon,
};

/// Non accessible area on the map.
///
/// An area is considered inaccessible if a centroid of a moving object cannot
/// move inside. A padding / offset around static objects is included in the
/// area to accommodate for non-zero moving object sizes and moving object
/// trajectory smoothing.
#[derive(Clone, Debug)]
pub(crate) struct ExclusionArea {
    polygon: ConvexPolygon,
    aabb: AABB,
}

impl ExclusionArea {
    /// Builds and returns a list of exclusion areas from an iterator of static
    /// object ichnographies and their world-to-object transforms.
    ///
    /// Each ichnography is offset by a padding.
    pub(crate) fn build(
        cache: impl IchnographyCache,
        objects: &[(GlobalTransform, ObjectType)],
    ) -> Vec<Self> {
        if objects.is_empty() {
            return Vec::new();
        }

        let mut max_extent: f32 = 1.;
        let exclusions: Vec<Self> = objects
            .iter()
            .map(|(transform, object_type)| {
                Self::from_ichnography(transform, cache.get_ichnography(*object_type))
            })
            .inspect(|exclusion| max_extent = max_extent.max(exclusion.aabb().extents().max()))
            .collect();
        Self::merge(exclusions, max_extent)
    }

    fn merge(mut exclusions: Vec<Self>, max_extent: f32) -> Vec<Self> {
        let mut merger = Merger::new(5. * max_extent);
        for exclusion in exclusions.drain(..) {
            let to_merge = merger.remove_intersecting(&exclusion);
            if to_merge.is_empty() {
                merger.insert(exclusion);
            } else {
                merger.insert(Self::merged(&exclusion, &to_merge));
            }
        }
        merger.into_vec()
    }

    /// Creates a new exclusion area from a static object ichnography and its
    /// world-to-object transform.
    fn from_ichnography(transform: &GlobalTransform, ichnography: &Ichnography) -> Self {
        let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
        let translation = transform.translation.to_flat();
        let isometry = Isometry::new(translation.into(), angle);
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
        Self { polygon, aabb }
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

    fn aabb(&self) -> &AABB {
        &self.aabb
    }

    fn intersects(&self, other: &ExclusionArea) -> bool {
        if !self.aabb.intersects(&other.aabb) {
            return false;
        }

        query::intersection_test(
            &Isometry::identity(),
            &self.polygon,
            &Isometry::identity(),
            &other.polygon,
        )
        .unwrap()
    }
}

/// A struct which holds exclusion areas in a rectangular grid of a given size.
struct Merger {
    tile_size: f32,
    grid: AHashMap<IVec2, Vec<ExclusionArea>>,
}

impl Merger {
    fn new(tile_size: f32) -> Self {
        Self {
            tile_size,
            grid: AHashMap::new(),
        }
    }

    fn insert(&mut self, exclusion: ExclusionArea) {
        let key = self.key(&exclusion);
        self.grid
            .entry(key)
            .or_insert_with(Vec::new)
            .push(exclusion);
    }

    fn remove_intersecting(&mut self, exclusion: &ExclusionArea) -> Vec<ExclusionArea> {
        let center_key = self.key(exclusion);
        let mut intersecting = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                let key = IVec2::new(center_key.x + dx, center_key.y + dy);
                if let Some(exclusions) = self.grid.get_mut(&key) {
                    for i in (0..exclusions.len()).rev() {
                        if exclusions[i].intersects(exclusion) {
                            intersecting.push(exclusions.swap_remove(i));
                        }
                    }
                }
            }
        }

        intersecting
    }

    fn into_vec(mut self) -> Vec<ExclusionArea> {
        self.grid.values_mut().flat_map(|v| v.drain(..)).collect()
    }

    fn key(&self, exclusion: &ExclusionArea) -> IVec2 {
        let center = exclusion.aabb().center();
        IVec2::new(
            (center.x / self.tile_size) as i32,
            (center.x / self.tile_size) as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;

    #[test]
    fn test_merge() {
        let transform_a = GlobalTransform {
            translation: Vec3::new(0., 0., -1.),
            ..Default::default()
        };
        let ichnography_a = Ichnography::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(-2., 5.),
                Point::new(-2., -3.),
                Point::new(4., -3.),
                Point::new(4., 5.),
            ])
            .unwrap(),
        );
        let transform_b = GlobalTransform::default();
        let ichnography_b = Ichnography::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(-1.5, 8.),
                Point::new(-1.5, 3.),
                Point::new(3.5, 3.),
                Point::new(3.5, 8.),
            ])
            .unwrap(),
        );
        let transform_c = GlobalTransform::default();
        let ichnography_c = Ichnography::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(20., 20.),
                Point::new(20., 18.),
                Point::new(25., 18.),
                Point::new(25., 20.),
            ])
            .unwrap(),
        );

        let exclusions = ExclusionArea::merge(
            vec![
                ExclusionArea::from_ichnography(&transform_a, &ichnography_a),
                ExclusionArea::from_ichnography(&transform_b, &ichnography_b),
                ExclusionArea::from_ichnography(&transform_c, &ichnography_c),
            ],
            7.,
        );
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

    #[test]
    fn test_merger() {
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
                Point::new(-10., -10.),
                Point::new(-10., -9.),
                Point::new(-9., -9.),
                Point::new(-9., -10.),
            ])
            .unwrap(),
        );
        let c = ExclusionArea::new(
            ConvexPolygon::from_convex_hull(&[
                Point::new(-1.5, -1.5),
                Point::new(-1.5, 0.5),
                Point::new(0.5, 0.5),
                Point::new(0.5, -1.5),
            ])
            .unwrap(),
        );

        let mut merger = Merger::new(4.);
        merger.insert(a.clone());
        merger.insert(b.clone());
        assert_eq!(merger.into_vec().len(), 2);

        let mut merger = Merger::new(4.);
        merger.insert(a);
        merger.insert(b);
        assert_eq!(merger.remove_intersecting(&c).len(), 1);
        assert_eq!(merger.into_vec().len(), 1);
    }
}
