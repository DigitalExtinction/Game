use anyhow::anyhow;
use parry2d::{bounding_volume::Aabb, math::Point, shape::ConvexPolygon};
use serde::{Deserialize, Serialize};

/// Padding around static object ichnographies used to accommodate for moving
/// object trajectory smoothing and non-zero moving object sizes.
pub const EXCLUSION_OFFSET: f32 = 2.;

pub struct Ichnography {
    radius: f32,
    local_aabb: Aabb,
    convex_hull: ConvexPolygon,
    offset_convex_hull: ConvexPolygon,
}

impl Ichnography {
    fn new(
        radius: f32,
        local_aabb: Aabb,
        convex_hull: ConvexPolygon,
        offset_convex_hull: ConvexPolygon,
    ) -> Self {
        Self {
            radius,
            local_aabb,
            convex_hull,
            offset_convex_hull,
        }
    }

    pub fn local_aabb(&self) -> Aabb {
        self.local_aabb
    }

    pub fn convex_hull(&self) -> &ConvexPolygon {
        &self.convex_hull
    }

    pub fn offset_convex_hull(&self) -> &ConvexPolygon {
        &self.offset_convex_hull
    }

    /// Returns minimum radius of a circle containing convex hull of the
    /// ichnography placed at local origin (0., 0.).
    pub fn radius(&self) -> f32 {
        self.radius
    }
}

impl From<ConvexPolygon> for Ichnography {
    fn from(footprint: ConvexPolygon) -> Self {
        // self.convex_hull.bounding_sphere() cannot be used since it assumes
        // different circle center.
        let radius = footprint
            .points()
            .iter()
            .map(|p| p.coords.norm())
            .max_by(f32::total_cmp)
            .unwrap();

        let local_aabb = footprint.local_aabb();
        let offset = footprint.offsetted(EXCLUSION_OFFSET);
        Self::new(radius, local_aabb, footprint, offset)
    }
}

impl TryFrom<FootprintSerde> for Ichnography {
    type Error = anyhow::Error;

    fn try_from(footprint: FootprintSerde) -> Result<Self, Self::Error> {
        Ok(ConvexPolygon::from_convex_polyline(
            footprint
                .convex_hull
                .iter()
                .map(|&[x, y]| Point::new(x, y))
                .collect(),
        )
        .ok_or_else(|| anyhow!("Polygon lies (almost) on a line."))?
        .into())
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FootprintSerde {
    convex_hull: Vec<[f32; 2]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radius() {
        let footpring = ConvexPolygon::from_convex_polyline(vec![
            Point::new(-10., 125.),
            Point::new(2., 125.),
            Point::new(2., 225.),
        ])
        .unwrap();
        let radius = Ichnography::from(footpring).radius();
        assert!((radius - 225.009).abs() < 0.01);
    }

    #[test]
    fn test_ichnography_from() {
        let footpring = ConvexPolygon::from_convex_polyline(vec![
            Point::new(15., 125.),
            Point::new(20., 125.),
            Point::new(20., 225.),
            Point::new(15., 225.),
        ])
        .unwrap();
        let ichnography = Ichnography::from(footpring);
        assert_eq!(
            ichnography.local_aabb(),
            Aabb::new(Point::new(15., 125.), Point::new(20., 225.),)
        );
        assert_eq!(
            ichnography.convex_hull().points(),
            &[
                Point::new(15., 125.),
                Point::new(20., 125.),
                Point::new(20., 225.),
                Point::new(15., 225.),
            ]
        );
        assert_eq!(
            ichnography.offset_convex_hull().points(),
            &[
                Point::new(13., 123.),
                Point::new(22., 123.),
                Point::new(22., 227.),
                Point::new(13., 227.),
            ]
        );
    }
}
