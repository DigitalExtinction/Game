#![allow(rustdoc::private_intra_doc_links)]
//! This module implements 2D object partitioning for fast geometric lookup,
//! for example ray casting.
//!
//! The core structure is a square tile grid which points to Bevy ECS entities.
//! Newly spawned entities are automatically added, despawned entities removed
//! and moved entities updated by systems added by
//! [`self::IndexPlugin`].
mod aabb;
mod collider;
mod grid;
mod index;
mod range;
mod segment;
mod systems;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use systems::IndexPlugin;

pub use self::{
    collider::{ColliderWithCache, LocalCollider, QueryCollider},
    index::{EntityIndex, RayEntityIntersection, SpatialQuery},
    systems::IndexLabel,
};

/// Size (in world-space) of a single square tile where entities are kept.
const TILE_SIZE: f32 = 10.;

pub struct IndexPluginGroup;

impl PluginGroup for IndexPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(IndexPlugin)
    }
}
