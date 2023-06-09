use ahash::AHashSet;
use bevy::prelude::Entity;
use parry3d::bounding_volume::Aabb;

use crate::{grid::TileGrid, range::TileRange};

/// An iterator over unique entity IDs withing a box.
pub(crate) struct AabbCandidates<'a> {
    grid: &'a TileGrid,
    tiles: TileRange,
    row: Option<i32>,
    prev_row: AHashSet<Entity>,
    current_row: AHashSet<Entity>,
}

impl<'a> AabbCandidates<'a> {
    /// Creates a new iterator of entities potentially colliding with a given
    /// AABB.
    pub(crate) fn new(grid: &'a TileGrid, aabb: &Aabb) -> Self {
        Self {
            grid,
            tiles: TileRange::from_aabb(aabb),
            row: None,
            prev_row: AHashSet::new(),
            current_row: AHashSet::new(),
        }
    }
}

impl<'a> Iterator for AabbCandidates<'a> {
    type Item = AHashSet<Entity>;

    fn next(&mut self) -> Option<AHashSet<Entity>> {
        loop {
            let tile_coords = match self.tiles.next() {
                Some(tile_coords) => tile_coords,
                None => return None,
            };

            let row = Some(tile_coords.y);
            if self.row != row {
                std::mem::swap(&mut self.prev_row, &mut self.current_row);
                self.current_row.clear();
                self.row = row;
            }

            if let Some(entities) = self.grid.get_tile_entities(tile_coords) {
                debug_assert!(!entities.is_empty());

                let mut new_entities = entities.to_owned();
                for entity in self.current_row.iter() {
                    new_entities.remove(entity);
                }
                self.current_row.extend(&new_entities);
                for entity in self.prev_row.iter() {
                    new_entities.remove(entity);
                }

                if !new_entities.is_empty() {
                    return Some(new_entities);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use parry3d::math::Point;

    use super::*;
    use crate::TILE_SIZE;

    #[test]
    fn test_aabb() {
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
        let entity_c = Entity::from_raw(3);
        let aabb_c = Aabb::new(
            Point::new(TILE_SIZE * 20.1, 0.5, TILE_SIZE * 20.5),
            Point::new(TILE_SIZE * 20., 0.5, TILE_SIZE * 20.2),
        );

        let mut grid = TileGrid::new();
        grid.insert(entity_a, &aabb_a);
        grid.insert(entity_b, &aabb_b);
        grid.insert(entity_c, &aabb_c);

        let mut candidates = AabbCandidates::new(
            &grid,
            &Aabb::new(
                Point::new(-TILE_SIZE * 0.1, 0.5, -TILE_SIZE * 5.5),
                Point::new(TILE_SIZE * 0.9, 0.5, TILE_SIZE * 1.2),
            ),
        );
        let first = candidates.next().unwrap();
        assert_eq!(first, AHashSet::from_iter(vec![entity_a]));
        let second = candidates.next().unwrap();
        assert_eq!(second, AHashSet::from_iter(vec![entity_b]));
        assert!(candidates.next().is_none());
    }
}
