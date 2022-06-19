use de_core::projection::ToFlat;
use glam::{IVec2, Vec2};
use parry3d::bounding_volume::AABB;

use crate::TILE_SIZE;

/// Iterable rectangular range of tiles.
///
/// The tiles are iterated row-by-row, for example: (1, 1) -> (2, 1) -> (1, 2)
/// -> (2, 2).
pub(crate) struct TileRange {
    a: IVec2,
    b: IVec2,
    x: i32,
    y: i32,
    exhausted: bool,
}

impl TileRange {
    /// Creates minimum tile range covers a given AABB.
    ///
    /// Tiles are assumed to be topologically closed. In other words, both
    /// touching and intersecting tiles are included in the range.
    pub(crate) fn from_aabb(aabb: &AABB) -> Self {
        let aabb = aabb.to_flat();
        let min_flat: Vec2 = aabb.mins.into();
        let max_flat: Vec2 = aabb.maxs.into();
        let start = (min_flat / TILE_SIZE).floor().as_ivec2();
        let stop = (max_flat / TILE_SIZE).floor().as_ivec2();
        Self::new(start, stop)
    }

    /// # Arguments
    ///
    /// * `a` - inclusive range start.
    ///
    /// * `b` - inclusive range end.
    pub(crate) fn new(a: IVec2, b: IVec2) -> Self {
        Self {
            a,
            b,
            x: a.x,
            y: a.y,
            exhausted: a.cmpgt(b).any(),
        }
    }

    /// Returns true if the given point is not contained in the tile range.
    pub(crate) fn excludes(&self, point: IVec2) -> bool {
        self.a.cmpgt(point).any() || self.b.cmplt(point).any()
    }

    /// Returns intersecting tile range. The result might be empty.
    pub(crate) fn intersection(&self, other: &TileRange) -> TileRange {
        Self::new(self.a.max(other.a), self.b.min(other.b))
    }
}

impl PartialEq for TileRange {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b
    }
}

impl Eq for TileRange {}

impl Iterator for TileRange {
    type Item = IVec2;

    fn next(&mut self) -> Option<IVec2> {
        if self.exhausted {
            return None;
        }

        let next = Some(IVec2::new(self.x, self.y));
        if self.x == self.b.x {
            if self.y == self.b.y {
                self.exhausted = true;
            } else {
                self.x = self.a.x;
                self.y += 1;
            }
        } else {
            self.x += 1;
        }
        next
    }
}
