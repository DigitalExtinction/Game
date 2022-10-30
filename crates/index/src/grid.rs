//! This module contains implementation of entity index based on 2D square grid
//! of entity sets.

use ahash::{AHashMap, AHashSet};
use bevy::prelude::Entity;
use glam::IVec2;
use parry3d::bounding_volume::Aabb;

use crate::range::TileRange;

/// Rectangular (2D) grid of sets of Bevy ECS entities.
///
/// Only non-empty sets are kept (a hash map mapping 2D tile coordinates to
/// Entity sets is used under the hood). Each set contains entities whose
/// absolute AABB intersects with the tile.
pub(crate) struct TileGrid {
    tiles: AHashMap<IVec2, AHashSet<Entity>>,
}

impl TileGrid {
    /// Creates a new empty grid.
    pub(crate) fn new() -> Self {
        Self {
            tiles: AHashMap::new(),
        }
    }

    /// Inserts an entity to the grid.
    ///
    /// # Arguments
    ///
    /// * `entity` - entity to be inserted to the grid.
    ///
    /// * `aabb` - world-space bounding box of the entity.
    ///
    /// # Panics
    ///
    /// Might panic if the entity is already present in the grid.
    pub(crate) fn insert(&mut self, entity: Entity, aabb: &Aabb) {
        for tile in TileRange::from_aabb(aabb) {
            self.insert_to_tile(entity, tile);
        }
    }

    /// Removes an entity from the grid.
    ///
    /// # Arguments
    ///
    /// * `entity` - entity to be removed from the grid.
    ///
    /// * `aabb` - world-space bounding box of the entity. The bounding box has
    ///   to be equal to the last bounding box used for insertion or update for
    ///   the entity.
    ///
    /// # Panics
    ///
    /// Might panic if the entity is not stored in the grid or if the last used
    /// update / insertion AABB differs from the one passed as an argument.
    pub(crate) fn remove(&mut self, entity: Entity, aabb: &Aabb) {
        for tile in TileRange::from_aabb(aabb) {
            self.remove_from_tile(entity, tile);
        }
    }

    /// Update bounding box of an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - entity to be updated.
    ///
    /// * `old_aabb` - bounding box used during insertion or last update of the
    ///   entity.
    ///
    /// * `new_aabb` - new world-space bounding box.
    ///
    /// # Panics
    ///
    /// Might panic if the entity is not present in the grid or if `old_aabb`
    /// differs from the last used update / insert AABB.
    pub(crate) fn update(&mut self, entity: Entity, old_aabb: &Aabb, new_aabb: &Aabb) {
        let old_tiles = TileRange::from_aabb(old_aabb);
        let new_tiles = TileRange::from_aabb(new_aabb);

        // Most of the time entities move withing the some tile range.
        if old_tiles == new_tiles {
            return;
        }

        let intersection = old_tiles.intersection(&new_tiles);
        for tile in old_tiles {
            if intersection.excludes(tile) {
                self.remove_from_tile(entity, tile);
            }
        }
        for tile in new_tiles {
            if intersection.excludes(tile) {
                self.insert_to_tile(entity, tile);
            }
        }
    }

    /// Returns entities intersecting a tile.
    ///
    /// Returns `None` if there are no entities intersecting the tile. Empty
    /// set is never returned.
    ///
    /// # Arguments
    ///
    /// `tile_coords` - coordinates of the tile.
    pub(crate) fn get_tile_entities(&self, tile_coords: IVec2) -> Option<&AHashSet<Entity>> {
        self.tiles.get(&tile_coords)
    }

    fn insert_to_tile(&mut self, entity: Entity, tile_coords: IVec2) {
        let inserted = self
            .tiles
            .entry(tile_coords)
            .or_insert_with(AHashSet::new)
            .insert(entity);
        debug_assert!(inserted);
    }

    fn remove_from_tile(&mut self, entity: Entity, tile_coords: IVec2) {
        let tile = self
            .tiles
            .get_mut(&tile_coords)
            .expect("Tried to remove an entity from a non-existent tile.");

        if tile.len() == 1 {
            let removed = self.tiles.remove(&tile_coords);
            debug_assert!(removed.is_some());
        } else {
            let removed = tile.remove(&entity);
            debug_assert!(removed);
        }
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use parry3d::math::Point;

    use super::*;
    use crate::TILE_SIZE;

    #[test]
    fn test_grid() {
        let mut grid = TileGrid::new();

        let entity_a = Entity::from_raw(1);
        let aabb_a = Aabb::new(
            Point::new(-TILE_SIZE * 0.5, -100.5, TILE_SIZE * 3.2),
            Point::new(-TILE_SIZE * 0.4, 3.5, TILE_SIZE * 3.5),
        );

        let entity_b = Entity::from_raw(2);
        let aabb_b = Aabb::new(
            Point::new(-TILE_SIZE * 0.7, -100.5, TILE_SIZE * 3.2),
            Point::new(-TILE_SIZE * 0.6, 3.5, TILE_SIZE * 3.5),
        );

        let entity_c = Entity::from_raw(3);
        let aabb_c_old = Aabb::new(
            Point::new(TILE_SIZE * 7., -100.5, -TILE_SIZE * 9.9),
            Point::new(TILE_SIZE * 8.5, 3.5, -TILE_SIZE * 8.),
        );
        let aabb_c_new = Aabb::new(
            Point::new(TILE_SIZE * 7.1, -100.5, -TILE_SIZE * 12.2),
            Point::new(TILE_SIZE * 8.5, 3.5, -TILE_SIZE * 9.3),
        );

        assert!(grid.get_tile_entities(IVec2::new(-1, -4)).is_none());

        grid.insert(entity_a, &aabb_a);
        assert_eq!(
            grid.get_tile_entities(IVec2::new(-1, -4)).unwrap(),
            &AHashSet::from_iter(vec![entity_a])
        );
        assert!(grid.get_tile_entities(IVec2::new(0, -4)).is_none());
        assert!(grid.get_tile_entities(IVec2::new(-2, -4)).is_none());
        assert!(grid.get_tile_entities(IVec2::new(-1, -5)).is_none());
        assert!(grid.get_tile_entities(IVec2::new(-1, -3)).is_none());

        grid.remove(entity_a, &aabb_a);
        assert!(grid.get_tile_entities(IVec2::new(-1, -4)).is_none());

        grid.insert(entity_a, &aabb_a);
        assert_eq!(
            grid.get_tile_entities(IVec2::new(-1, -4)).unwrap(),
            &AHashSet::from_iter(vec![entity_a])
        );

        grid.insert(entity_b, &aabb_b);
        grid.insert(entity_c, &aabb_c_old);
        assert_eq!(
            grid.get_tile_entities(IVec2::new(-1, -4)).unwrap(),
            &AHashSet::from_iter(vec![entity_a, entity_b])
        );
        assert_eq!(
            grid.get_tile_entities(IVec2::new(7, 8)).unwrap(),
            &AHashSet::from_iter(vec![entity_c])
        );
        assert_eq!(
            grid.get_tile_entities(IVec2::new(7, 9)).unwrap(),
            &AHashSet::from_iter(vec![entity_c])
        );
        assert_eq!(
            grid.get_tile_entities(IVec2::new(8, 9)).unwrap(),
            &AHashSet::from_iter(vec![entity_c])
        );

        grid.update(entity_c, &aabb_c_old, &aabb_c_new);
        assert!(grid.get_tile_entities(IVec2::new(7, 8)).is_none());
        assert_eq!(
            grid.get_tile_entities(IVec2::new(8, 9)).unwrap(),
            &AHashSet::from_iter(vec![entity_c])
        );
        assert_eq!(
            grid.get_tile_entities(IVec2::new(8, 12)).unwrap(),
            &AHashSet::from_iter(vec![entity_c])
        );

        grid.remove(entity_a, &aabb_a);
        grid.remove(entity_b, &aabb_a);
        grid.remove(entity_c, &aabb_c_new);
        for x in -20..20 {
            for y in -20..20 {
                assert!(grid.get_tile_entities(IVec2::new(x, y)).is_none());
            }
        }
    }

    #[test]
    fn test_tile_range_from_aabb() {
        let aabb = Aabb::new(
            Point::new(-TILE_SIZE * 0.5, -100.5, -TILE_SIZE * 4.5),
            Point::new(TILE_SIZE * 1., 3.5, -TILE_SIZE * 3.5),
        );
        let tiles: Vec<IVec2> = TileRange::from_aabb(&aabb).collect();
        assert_eq!(
            tiles,
            vec![
                IVec2::new(-1, 3),
                IVec2::new(0, 3),
                IVec2::new(1, 3),
                IVec2::new(-1, 4),
                IVec2::new(0, 4),
                IVec2::new(1, 4),
            ]
        );
    }

    #[test]
    fn test_tile_range() {
        let negative: Vec<IVec2> = TileRange::new(IVec2::new(-1, 2), IVec2::new(0, 4)).collect();
        assert_eq!(
            negative,
            vec![
                IVec2::new(-1, 2),
                IVec2::new(0, 2),
                IVec2::new(-1, 3),
                IVec2::new(0, 3),
                IVec2::new(-1, 4),
                IVec2::new(0, 4),
            ]
        );

        let mut empty = TileRange::new(IVec2::new(-1, 2), IVec2::new(-2, 4));
        assert!(empty.next().is_none());
    }

    #[test]
    fn test_tile_range_excludes() {
        let range = TileRange::new(IVec2::new(-4, -7), IVec2::new(-2, -6));
        assert!(!range.excludes(IVec2::new(-4, -7)));
        assert!(!range.excludes(IVec2::new(-2, -6)));
        assert!(range.excludes(IVec2::new(-5, -7)));
        assert!(range.excludes(IVec2::new(-1, -7)));
        assert!(range.excludes(IVec2::new(-4, -8)));
        assert!(range.excludes(IVec2::new(-4, 1)));
    }

    #[test]
    fn test_tile_range_intersection() {
        let range = TileRange::new(IVec2::new(10, 12), IVec2::new(20, 22));

        let intersection: Vec<IVec2> = range
            .intersection(&TileRange::new(IVec2::new(20, 12), IVec2::new(20, 13)))
            .collect();
        assert_eq!(intersection, vec![IVec2::new(20, 12), IVec2::new(20, 13)]);

        let intersection: Vec<IVec2> = range
            .intersection(&TileRange::new(IVec2::new(500, 500), IVec2::new(600, 600)))
            .collect();
        assert_eq!(intersection, vec![]);
    }
}
