//! Tools for retrieving entities potentially intersecting a line segment.

use ahash::AHashSet;
use bevy::prelude::Entity;
use de_types::projection::ToFlat;
use glam::{IVec2, Vec2};
use parry3d::shape::Segment;

use super::{grid::TileGrid, TILE_SIZE};

/// An iterator over sets of entities from tiles intersecting a given line
/// segment.
///
/// This iterator yields non-empty sets. Each yielded set contains all entities
/// from a grid tile which have not yet been present in any of previously
/// yielded sets. In other words, the yielded sets are disjoint.
///
/// The tiles (and thus the yielded sets) are iterated by increasing distance
/// between point `a` of the given line segment and the intersection of the
/// tile with the line segment.
pub(crate) struct SegmentCandidates<'a> {
    grid: &'a TileGrid,
    tiles: TileIterator,
    encountered: Option<&'a AHashSet<Entity>>,
}

impl<'a> SegmentCandidates<'a> {
    pub(crate) fn new(grid: &'a TileGrid, segment: Segment) -> Self {
        Self {
            grid,
            tiles: TileIterator::new(segment),
            encountered: None,
        }
    }
}

impl<'a> Iterator for SegmentCandidates<'a> {
    type Item = AHashSet<Entity>;

    fn next(&mut self) -> Option<AHashSet<Entity>> {
        loop {
            let tile_coords = match self.tiles.next() {
                Some(tile_coords) => tile_coords,
                None => return None,
            };

            match self.grid.get_tile_entities(tile_coords) {
                Some(entities) => {
                    debug_assert!(!entities.is_empty());

                    // All entities are inserted to tiles intersecting their
                    // AABB, thus the set of insertion tiles (of a given
                    // entity) forms a square. Squares are convex, therefore,
                    // filtering solely by previously visited tile is
                    // sufficient to remove duplicates.
                    let new_entities = match self.encountered {
                        Some(encountered) => entities.difference(encountered).cloned().collect(),
                        None => entities.clone(),
                    };
                    self.encountered = Some(entities);
                    if !new_entities.is_empty() {
                        return Some(new_entities);
                    }
                }
                None => self.encountered = None,
            }
        }
    }
}

/// Iterator over tiles intersecting a line segment.
struct TileIterator {
    point: Vec2,
    stop: Vec2,
    last_tile: IVec2,
    finished: bool,
}

impl TileIterator {
    /// Creates a new iterator from a given line segment.
    ///
    /// # Arguments
    ///
    /// * `segment` - a 2D line segment is created from orthographic projection
    ///   of this 3D line segment onto the map surface.
    fn new(segment: Segment) -> Self {
        let mut point = segment.a.to_flat();
        let stop = segment.b.to_flat();

        if point != stop {
            // First tile might be duplicated if direction is negative along
            // any axis. The following code fixes the issue.
            let next_point = Self::next_point(point, stop);
            if (next_point / TILE_SIZE).floor() == (point / TILE_SIZE).floor() {
                point = next_point;
            }
        }

        Self {
            point,
            stop,
            last_tile: (stop / TILE_SIZE).floor().as_ivec2(),
            finished: false,
        }
    }

    fn next_point(point: Vec2, stop: Vec2) -> Vec2 {
        let dir = stop - point;
        debug_assert!(dir != Vec2::ZERO);

        let current_tile_float = point / TILE_SIZE;
        let next_tile_x = TILE_SIZE
            * if dir.x >= 0. {
                current_tile_float.x.floor() + 1.
            } else {
                current_tile_float.x.ceil() - 1.
            };
        let next_tile_y = TILE_SIZE
            * if dir.y >= 0. {
                current_tile_float.y.floor() + 1.
            } else {
                current_tile_float.y.ceil() - 1.
            };

        let factor_x = if dir.x == 0. {
            f32::INFINITY
        } else {
            (next_tile_x - point.x) / dir.x
        };
        let factor_y = if dir.y == 0. {
            f32::INFINITY
        } else {
            (next_tile_y - point.y) / dir.y
        };

        if factor_x < factor_y {
            if factor_x >= 1. {
                // Avoid rounding issues near the target point.
                stop
            } else {
                Vec2::new(next_tile_x, point.y + factor_x * dir.y)
            }
        } else if factor_y >= 1. {
            // Avoid rounding issues near the target point.
            stop
        } else {
            Vec2::new(point.x + factor_y * dir.x, next_tile_y)
        }
    }
}

impl Iterator for TileIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<IVec2> {
        if self.finished {
            return None;
        }

        let current_tile = (self.point / TILE_SIZE).floor().as_ivec2();
        if current_tile == self.last_tile {
            self.finished = true;
        } else {
            self.point = Self::next_point(self.point, self.stop);
        }
        Some(current_tile)
    }
}

#[cfg(test)]
mod tests {
    use parry3d::{bounding_volume::Aabb, math::Point, shape::Segment};

    use super::*;
    use crate::grid::TileGrid;

    #[test]
    fn test_segment_candidates() {
        let entity_a = Entity::from_raw(1);
        let aabb_a = Aabb::new(
            Point::new(0.5 * TILE_SIZE, 0., 1.1 * TILE_SIZE),
            Point::new(3.7 * TILE_SIZE, 3., 1.6 * TILE_SIZE),
        );

        let entity_b = Entity::from_raw(2);
        let aabb_b = Aabb::new(
            Point::new(-TILE_SIZE * 0.7, -100.5, -TILE_SIZE * 3.5),
            Point::new(-TILE_SIZE * 0.6, 3.5, -TILE_SIZE * 3.2),
        );

        let mut grid = TileGrid::new();
        grid.insert(entity_a, &aabb_a);
        grid.insert(entity_b, &aabb_b);

        let segment = Segment::new(
            Point::new(0.2 * TILE_SIZE, 0., 1.2 * TILE_SIZE),
            Point::new(1.1 * TILE_SIZE, 0., 1000. * TILE_SIZE),
        );

        let mut candidates = SegmentCandidates::new(&grid, segment);
        let first = candidates.next().unwrap();
        assert_eq!(first, AHashSet::from_iter(vec![entity_a]));
        assert!(candidates.next().is_none());
    }

    #[test]
    fn test_tile_iterator() {
        // (-3, -4) -> (1, 2)
        let xy = Segment::new(
            Point::new(-2. * TILE_SIZE, 0., 3.1 * TILE_SIZE),
            Point::new(1.1 * TILE_SIZE, 0., -2.7 * TILE_SIZE),
        );
        let xy_neg = Segment::new(
            Point::new(1.1 * TILE_SIZE, 0., -2.7 * TILE_SIZE),
            Point::new(-2. * TILE_SIZE, 0., 3.1 * TILE_SIZE),
        );

        let tiles: Vec<IVec2> = TileIterator::new(xy).collect();
        assert_eq!(
            tiles,
            vec![
                IVec2::new(-2, -4),
                IVec2::new(-2, -3),
                IVec2::new(-2, -2),
                IVec2::new(-1, -2),
                IVec2::new(-1, -1),
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(0, 1),
                IVec2::new(0, 2),
                IVec2::new(1, 2),
            ]
        );

        let tiles_neg: Vec<IVec2> = TileIterator::new(xy_neg).collect();
        assert_eq!(
            tiles_neg,
            vec![
                IVec2::new(1, 2),
                IVec2::new(0, 2),
                IVec2::new(0, 1),
                IVec2::new(0, 0),
                IVec2::new(-1, 0),
                IVec2::new(-1, -1),
                IVec2::new(-1, -2),
                IVec2::new(-2, -2),
                IVec2::new(-2, -3),
                IVec2::new(-2, -4),
            ]
        );
    }

    #[test]
    fn test_tile_iterator_sort_empty() {
        let short = Segment::new(
            Point::new(1.1 * TILE_SIZE, 0., -3.1 * TILE_SIZE),
            Point::new(1.2 * TILE_SIZE, 0., -3.1 * TILE_SIZE),
        );
        let tiles_short: Vec<IVec2> = TileIterator::new(short).collect();
        assert_eq!(tiles_short, vec![IVec2::new(1, 3)]);

        let empty = Segment::new(
            Point::new(0.1 * TILE_SIZE, 0., -3.1 * TILE_SIZE),
            Point::new(0.1 * TILE_SIZE, 0., -3.1 * TILE_SIZE),
        );
        let tiles_empty: Vec<IVec2> = TileIterator::new(empty).collect();
        assert_eq!(tiles_empty, vec![IVec2::new(0, 3)]);
    }
}
