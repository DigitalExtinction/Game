use glam::IVec2;
use tinyvec::ArrayVec;

use super::{
    edge::{Edge, IntersectionDir},
    parameters::Transition,
};

/// Velocity region is an area enclosed by two or three edges.
pub(super) struct Region {
    edges: ArrayVec<[Edge; 3]>,
}

impl Region {
    /// Returns a new velocity region.
    ///
    /// # Arguments
    ///
    /// * `edges` - two or three edges enclosing the area.
    pub(super) fn new(edges: ArrayVec<[Edge; 3]>) -> Self {
        debug_assert!(edges.len() >= 2);
        Self { edges }
    }

    pub(super) fn edges(&self) -> &[Edge] {
        self.edges.as_slice()
    }

    /// Returns up to two intersection parameters ([`Transition`]) of an edge
    /// with the boundaries of the region.
    ///
    /// # Arguments
    ///
    /// * `edge` - an edge whose intersections with `self` are calculated.
    ///
    /// * `point_inside` - whether the point of the line associated with `edge`
    ///   is inside (boundary is inclusive) of `self`.
    pub(super) fn intersections(
        &self,
        edge: &Edge,
        point_inside: bool,
    ) -> ArrayVec<[Transition; 2]> {
        let mut intersections: ArrayVec<[Transition; 2]> = ArrayVec::new();
        for region_edge in self.edges.as_slice() {
            if let Some(intersection) = region_edge.intersection(&edge, point_inside) {
                let delta = match intersection.dir() {
                    IntersectionDir::InOut => -1,
                    IntersectionDir::OutIn => 1,
                };

                if intersections.iter().all(|t| t.delta() != delta) {
                    intersections.push(Transition::new(intersection.parameter(), delta));
                }
            }
        }

        intersections
    }

    /// Returns true if `point` is inside `self`.
    pub(super) fn contains(&self, point: IVec2) -> bool {
        self.edges.as_slice().iter().all(|e| e.inner_side(point))
    }
}
