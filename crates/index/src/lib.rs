//! This module implements 2D object partitioning for fast geometric lookup,
//! for example ray casting.
//!
//! The core structure is a square tile grid which points to Bevy ECS entities.
//! Newly spawned entities are automatically added, despawned entities removed
//! and moved entities updated by systems added by
//! [`self::IndexPlugin`].
mod grid;
mod index;
mod segment;
mod shape;
mod systems;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use systems::IndexPlugin;

pub use self::{
    index::{EntityIndex, RayEntityIntersection, SpatialQuery},
    shape::EntityShape,
};

/// Size (in world-space) of a single square tile where entities are kept.
const TILE_SIZE: f32 = 10.;

pub struct IndexPluginGroup;

impl PluginGroup for IndexPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(IndexPlugin);
    }
}
